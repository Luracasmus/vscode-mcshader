use std::result;

use super::*;

impl MinecraftLanguageServer {
    pub(super) fn collect_memory(&self, workspace_files: &mut HashMap<Rc<PathBuf>, Rc<WorkspaceFile>>) {
        workspace_files.retain(|_file_path, workspace_file| {
            // Only delete file that both do not exist and no file includes it.
            *workspace_file.file_type().borrow() != FileType::Invalid || !workspace_file.included_files().borrow().is_empty()
        });
    }

    pub(super) fn is_valid_shader<'a>(&'a self, shader_packs: &'a HashSet<Rc<ShaderPack>>, file_path: &Path) -> Option<&'a Rc<ShaderPack>> {
        for shader_pack in shader_packs {
            if let Ok(relative_path) = file_path.strip_prefix(&shader_pack.path) {
                let relative_path = relative_path.to_str().unwrap();
                if RE_BASIC_SHADERS.is_match(relative_path) {
                    return Some(shader_pack);
                } else if let Some(result) = relative_path.split_once(MAIN_SEPARATOR)
                    && RE_DIMENSION_FOLDER.is_match(result.0)
                    && RE_BASIC_SHADERS.is_match(result.1)
                {
                    return Some(shader_pack);
                }
                return None;
            }
        }
        None
    }

    pub(super) fn find_shader_packs(shader_packs: &mut Vec<Rc<ShaderPack>>, curr_path: PathBuf) {
        let file_name = curr_path.file_name().unwrap();
        if file_name == "shaders" {
            info!("Find shader pack {}", curr_path.to_str().unwrap());
            let debug = curr_path
                .parent()
                .and_then(|parent| parent.file_name())
                .is_some_and(|name| name == "debug");
            shader_packs.push(Rc::new(ShaderPack { path: curr_path, debug }));
        } else if file_name.to_str().is_none_or(|name| !name.starts_with('.') || name == ".minecraft")
            && let Ok(dir) = curr_path.read_dir()
        {
            dir.filter_map(result::Result::ok)
                .filter(|file| file.file_type().unwrap().is_dir())
                .for_each(|file| {
                    Self::find_shader_packs(shader_packs, file.path());
                });
        }
    }

    pub(super) fn scan_files_in_root(
        &self, parser: &mut Parser, shader_packs: &mut HashSet<Rc<ShaderPack>>,
        workspace_files: &mut HashMap<Rc<PathBuf>, Rc<WorkspaceFile>>, temp_files: &mut HashMap<PathBuf, TempFile>, root: PathBuf,
    ) {
        info!("Generating file framework on workspace \"{}\"", root.to_str().unwrap());

        let mut sub_shader_packs: Vec<Rc<ShaderPack>> = vec![];
        Self::find_shader_packs(&mut sub_shader_packs, root);

        for shader_pack in &sub_shader_packs {
            if let Ok(dir) = shader_pack.path.read_dir() {
                dir.filter_map(result::Result::ok).for_each(|file| {
                    let file_path = file.path();
                    if file.file_type().unwrap().is_file() {
                        if RE_BASIC_SHADERS.is_match(file.file_name().to_str().unwrap()) {
                            WorkspaceFile::new_shader(workspace_files, temp_files, parser, shader_pack, file_path);
                        }
                    } else if RE_DIMENSION_FOLDER.is_match(file.file_name().to_str().unwrap()) {
                        file_path.read_dir().unwrap().filter_map(result::Result::ok).for_each(|dim_file| {
                            let dim_file_path = dim_file.path();
                            if dim_file.file_type().unwrap().is_file() && RE_BASIC_SHADERS.is_match(dim_file.file_name().to_str().unwrap())
                            {
                                WorkspaceFile::new_shader(workspace_files, temp_files, parser, shader_pack, dim_file_path);
                            }
                        });
                    }
                });
            }
        }

        shader_packs.extend(sub_shader_packs);
    }

    pub(super) fn lint_workspace_shader(
        &self, shader_file: &ShaderData, shader_path: &Rc<PathBuf>, update_list: &mut HashMap<Rc<PathBuf>, Rc<WorkspaceFile>>,
    ) {
        let mut file_list = HashMap::new();
        let mut shader_content = String::new();
        let mut version = String::new();
        shader_file.0.merge_file(
            &mut file_list,
            &shader_file.0,
            &mut shader_content,
            &mut version,
            shader_path,
            &mut -1,
            0,
        );
        let offset = preprocess_shader(&mut shader_content, version, shader_file.0.shader_pack().debug);

        let shader_path_str = shader_path.to_str().unwrap();

        let mut cache = shader_file.0.cache().borrow_mut();
        let cache = cache.as_mut().unwrap();
        let hit_cache = cache.check(&shader_content);
        let validation_result = if hit_cache {
            None
        } else {
            crate::opengl::validate_shader(shader_file.0.file_type().borrow().to_shader_kind(), &shader_content)
        };

        if let Some(error) = validation_result {
            info!("Compilation errors reported; shader file: {shader_path_str},\nerrors: \"\n{error}\"");

            // We have ensured files in file lists are unique, so each file.diagnostics will exist only once
            // parent_shaders itself will not changed during parsing, this should be safe.
            let mut diagnostic_pointers = file_list
                .into_iter()
                .map(|(file_path, (index, workspace_file))| {
                    let pointer;
                    {
                        let parent_shaders = workspace_file.parent_shaders().borrow();
                        let mut diagnostic = parent_shaders.get(shader_path).unwrap().1.borrow_mut();
                        diagnostic.clear();
                        pointer = &mut *diagnostic as *mut Vec<Diagnostic>;
                    }
                    update_list.insert(file_path, workspace_file);
                    (index, pointer)
                })
                .collect::<HashMap<_, _>>();

            let message = match error {
                shaderc::Error::CompilationError(_, message) => message, // TODO: Handle the error code.
                shaderc::Error::ParseError(message) => message,
                _ => {
                    panic!();
                    //warn!("Non-shader glslang error encountered");
                    //String::new()
                }
            };

            message
                .split_terminator('\n')
                .filter_map(|log_line| DIAGNOSTICS_REGEX.captures(log_line))
                .for_each(|captures| {
                    let mut msg = captures.name("output").unwrap().as_str().trim().to_owned();

                    if msg != "compilation terminated" {
                        msg += ", from file: ";
                        msg += shader_path_str;

                        let line = captures
                            .name("linenum")
                            .map_or(0, |c| c.as_str().parse::<u32>().unwrap_or(0))
                            .saturating_sub(offset);

                        // We assume the current file is the file with the errors. This isn't always true. TODO: fix
                        let (line_offset, line_end) = if let Some(shader_line) = shader_file.0.content().borrow().lines().nth(line as usize)
                        {
                            let item = captures.name("item").unwrap().as_str();
                            let item_len = item.len();
                            let item_exists = item_len > 0;

                            if item_exists && let Some(start_bytes) = shader_line.find(item) {
                                let end_bytes = start_bytes + item_len;

                                (
                                    shader_line[..start_bytes].encode_utf16().count() as u32,
                                    shader_line[..end_bytes].encode_utf16().count() as u32,
                                )
                            } else {
                                if item_exists {
                                    msg = format!("'{item}': {msg}");
                                }

                                /*
                                let mut line_end_utf16 = if item_len > 0 {
                                    shader_line[..line_offset_bytes as usize + item_len].encode_utf16().count() as u32
                                } else {
                                    shader_line
                                        .split_once("")
                                        .map(|(prefix, _)| prefix)
                                        .or_else(|| shader_line.split_once("").map(|(prefix, _)| prefix))
                                        .map_or(u32::MAX, |prefix| prefix.trim_end().encode_utf16().count() as u32)
                                };

                                line_end_utf16 = item_len as u32;

                                (line_offset_utf16, line_end_utf16)
                                */
                                (0, u32::MAX) // TODO
                            }
                        } else {
                            (0, u32::MAX)
                        };

                        let severity = captures.name("severity").map_or(DiagnosticSeverity::INFORMATION, |c| {
                            match c.as_str().to_lowercase().as_str() {
                                "error" => DiagnosticSeverity::ERROR,
                                "warning" => DiagnosticSeverity::WARNING,
                                _ => DiagnosticSeverity::INFORMATION,
                            }
                        });

                        let diagnostic = Diagnostic {
                            range: Range {
                                start: Position {
                                    line,
                                    character: line_offset,
                                },
                                end: Position { line, character: line_end },
                            },
                            severity: Some(severity),
                            source: Some("mcshader-glsl".to_owned()),
                            message: msg,
                            ..Default::default()
                        };

                        let index = captures.name("filepath").unwrap();
                        if let Some(diagnostics) = diagnostic_pointers.get_mut(index.as_str()) {
                            unsafe { diagnostics.as_mut().unwrap().push(diagnostic) };
                        }
                    }
                });
        } else {
            if !hit_cache {
                cache.insert(&shader_content);
            }
            info!("Compilation reported no errors"; "shader file" => shader_path_str);
            file_list.into_iter().for_each(|(file_path, (_, workspace_file))| {
                workspace_file
                    .parent_shaders()
                    .borrow()
                    .get(shader_path)
                    .unwrap()
                    .1
                    .borrow_mut()
                    .clear();
                update_list.insert(file_path, workspace_file);
            });
        }
    }

    pub(super) fn lint_temp_file(&self, temp_file: &TempFile, file_path: &Path, url: Url, temp_lint: bool) -> Diagnostics {
        let diagnostics = if let Some((mut source, version)) = temp_file.merge_self(file_path) {
            let file_type = temp_file.file_type().borrow();
            let offset = preprocess_shader(&mut source, version.clone(), temp_file.shader_pack().debug);

            let mut cache = temp_file.cache().borrow_mut();
            let cache = cache.as_mut().unwrap();
            let hit_cache = cache.check(&source);

            let validation_result = if hit_cache {
                None
            } else {
                crate::opengl::validate_shader(file_type.to_shader_kind(), &source)
            };
            if let Some(compile_log) = validation_result {
                info!(
                    "Compilation errors reported; shader file: {},\nerrors: \"\n{compile_log}\"",
                    file_path.to_str().unwrap(),
                );
                /*compile_log
                .split_terminator('\n')
                .filter_map(|log_line| DIAGNOSTICS_REGEX.captures(log_line))
                .filter(|captures| captures.name("filepath").unwrap().as_str() == "0")
                .map(|captures| {
                    let msg = captures.name("output").unwrap().as_str().to_owned();

                    let line = captures
                        .name("linenum")
                        .map_or(0, |c| c.as_str().parse::<u32>().unwrap_or(0))
                        .saturating_sub(offset);

                    let (line_offset, line_end) = if let Some(shader_line) = temp_file.content().borrow().lines().nth(line as usize) {
                        let line_offset = captures.name("lineoffset").map_or(0, |c| {
                            c.as_str().parse::<u32>().map_or(0, |line_offset| {
                                shader_line
                                    .get(..(line_offset as usize))
                                    .map_or(0, |slice| slice.encode_utf16().count() as u32)
                            })
                        });

                        let line_end = shader_line
                            .split_once("")
                            .map(|(prefix, _)| prefix)
                            .or_else(|| shader_line.split_once("").map(|(prefix, _)| prefix))
                            .map_or(u32::MAX, |prefix| prefix.trim_end().encode_utf16().count() as u32);

                        (line_offset, line_end)
                    } else {
                        (0, u32::MAX)
                    };

                    let severity = captures.name("severity").map_or(DiagnosticSeverity::INFORMATION, |c| {
                        match c.as_str().to_lowercase().as_str() {
                            "error" => DiagnosticSeverity::ERROR,
                            "warning" => DiagnosticSeverity::WARNING,
                            _ => DiagnosticSeverity::INFORMATION,
                        }
                    });

                    Diagnostic {
                        range: Range {
                            start: Position {
                                line,
                                character: line_offset,
                            },
                            end: Position { line, character: line_end },
                        },
                        severity: Some(severity),
                        source: Some("mcshader-glsl".to_owned()),
                        message: msg,
                        ..Default::default()
                    }
                })
                .collect::<Vec<_>>()*/
                vec![]
            } else {
                if !hit_cache {
                    cache.insert(&source);
                }
                info!("Compilation reported no errors"; "shader file" => file_path.to_str().unwrap());
                vec![]
            }
        } else if temp_lint {
            TreeParser::simple_lint(
                &temp_file.tree().borrow(),
                &temp_file.content().borrow(),
                &temp_file.line_mapping().borrow(),
            )
        } else {
            vec![]
        };
        HashMap::from([(url, diagnostics)])
    }

    pub(super) fn collect_diagnostics(&self, update_list: &HashMap<Rc<PathBuf>, Rc<WorkspaceFile>>) -> Diagnostics {
        update_list
            .into_iter()
            .map(|(file_path, workspace_file)| {
                let file_url = Url::from_file_path(file_path as &Path).unwrap();
                let diagnostics = workspace_file
                    .parent_shaders()
                    .borrow()
                    .values()
                    .flat_map(|(_, diagnostics)| diagnostics.borrow().clone())
                    .collect::<Vec<_>>();
                (file_url, diagnostics)
            })
            .collect()
    }

    pub(super) fn initial_scan(&self, roots: Vec<PathBuf>) {
        let server_data = self.server_data.lock().unwrap();
        let mut parser = server_data.tree_sitter_parser.borrow_mut();
        let mut shader_packs = server_data.shader_packs.borrow_mut();
        let mut workspace_files = server_data.workspace_files.borrow_mut();
        let mut temp_files = server_data.temp_files.borrow_mut();

        for root in roots {
            self.scan_files_in_root(&mut parser, &mut shader_packs, &mut workspace_files, &mut temp_files, root);
        }
    }
}
