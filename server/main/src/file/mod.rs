use std::{
    cell::RefCell,
    ffi::OsString,
    fs::read_to_string,
    path::{Component, MAIN_SEPARATOR_STR, Path, PathBuf},
    rc::Rc,
};

use hashbrown::HashMap;
use itoa::Buffer;
use logging::{error, warn};
use regex::Matches;
use shaderc::ShaderKind;
use tower_lsp::lsp_types::*;
use tree_sitter::{InputEdit, Parser, Point, Tree};

use crate::constant::*;

mod compile_cache;
mod temp_file;
mod workspace_file;

pub type IncludeInformation = (usize, usize, usize, Rc<PathBuf>, Rc<WorkspaceFile>);
pub type ShaderData = (Rc<WorkspaceFile>, RefCell<Vec<Diagnostic>>);

/// Used to store comment type of multi line comments for ignored lines
#[derive(Clone, Copy, Eq, PartialEq, Hash)]
enum CommentType {
    None,
    Single,
    Multi,
}

fn include_path_join(root_path: &Path, curr_path: &Path, additional: &str) -> Result<PathBuf, &'static str> {
    let mut buffer: Vec<Component>;
    let additional = if let Some(path) = additional.strip_prefix('/') {
        buffer = root_path.components().collect();
        Path::new(path)
    } else {
        buffer = curr_path.components().collect();
        buffer.pop();
        Path::new(additional)
    };

    for component in additional.components() {
        match component {
            Component::ParentDir => {
                if let Some(Component::Normal(_)) = buffer.pop() {
                } else {
                    return Err("Unable to find parent folder while creating include path");
                }
            }
            Component::Normal(_) => buffer.push(component),
            Component::CurDir => (),
            _ => return Err("Invalid component in include path"),
        }
    }

    let mut resource = OsString::new();
    let last = buffer.pop().unwrap();
    for component in buffer {
        resource.push(component);
        match component {
            Component::Prefix(_) | Component::RootDir => {}
            _ => resource.push(MAIN_SEPARATOR_STR),
        }
    }
    resource.push(last);

    Ok(PathBuf::from(resource))
}

fn push_line_macro(content: &mut String, line: usize, file_id: &str, file_name: &str) {
    content.push_str("#line ");
    content.push_str(Buffer::new().format(line));
    content.push(' ');
    content.push_str(file_id);
    content.push_str("\t// ");
    content.push_str(file_name);
}

#[must_use]
pub fn generate_line_mapping(content: &str) -> Vec<usize> {
    let mut line_mapping = vec![0];
    content.match_indices('\n').for_each(|(index, _)| {
        line_mapping.push(index + 1);
    });
    line_mapping.push(content.len() + 1);
    line_mapping
}

fn push_str_without_ignored(
    shader_content: &mut String, file_content: &str, mut start_index: usize, end_index: usize, curr_line: usize,
    ignored_lines: &mut core::slice::Iter<'_, (usize, CommentType)>, line_mapping: &[usize],
) {
    for (line, comment_type) in ignored_lines.by_ref() {
        if *line > curr_line {
            break;
        }
        let line_start = line_mapping[*line];
        shader_content.push_str(&file_content[start_index..line_start]);
        match comment_type {
            CommentType::None => {}
            CommentType::Single => shader_content.push_str(r"// \"),
            CommentType::Multi => shader_content.push_str(r"/*"),
        }
        start_index = line_mapping[*line + 1] - 1;
    }
    shader_content.push_str(&file_content[start_index..end_index]);
}

#[must_use]
fn byte_offset(content: &str, chars: usize) -> usize {
    content[..chars].encode_utf16().count()
}

/// Byte index generated from char index
/// Returns (total_index, line_index)
#[must_use]
pub fn byte_index(content: &str, position: Position, line_mapping: &[usize]) -> (usize, usize) {
    let line_start = line_mapping[position.line as usize];
    let rest_content = &content[line_start..];
    let line_offset = byte_offset(rest_content, position.character as usize);
    (line_start + line_offset, line_offset)
}

fn end_in_comment(index: usize, comment_matches: Matches<'_, '_>, in_comment: &mut bool, comment_type: &mut bool) {
    for comment_match in comment_matches {
        if comment_match.start() < index {
            continue;
        }
        match comment_match.as_str() {
            "/*" => {
                // if `comment_type` is set to false, this line should be considerd as a single line comment
                if !*in_comment && *comment_type {
                    *in_comment = true;
                    *comment_type = true;
                }
            }
            "*/" => {
                *in_comment = false;
            }
            "//" => {
                // `//` would not make next line comment unless this line ends with `\`
                *comment_type &= *in_comment;
            }
            // `\$` for multi comment lines using `//`
            // This is the end of line so nothing left
            _ => {
                *in_comment |= !*comment_type;
            }
        }
    }
    // Reset comment_type to true if next line is not comment
    if !*in_comment {
        *comment_type = true;
    }
}

pub fn preprocess_shader(shader_content: &mut String, mut version: String) {
    // TODO: Patch `#extension` somehow so that it works like in Iris.

    if let Some(capture) = RE_MACRO_VERSION.captures(&version.clone()) {
        let version_capture = capture.get(1).unwrap();

        if version_capture.as_str().parse::<u32>().unwrap() < 330 {
            let range = version_capture.range();
            version.replace_range(range, "330"); // Patch up to 330 for shaderc.
        }
        // Ignore the possible multi-line comment start. It will be added on its original place later.
        version.truncate(capture.get(0).unwrap().end());
    }
    version = version.replace("compatibility", "core");
    version.push('\n');

    version += shader_content;
    *shader_content = version;
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum FileType {
    None,
    Invalid,
    Shader(ShaderKind),
}

impl FileType {
    #[must_use]
    pub const fn to_shader_kind(self) -> ShaderKind {
        let Self::Shader(shader_kind) = self else { panic!() };

        shader_kind
    }
}

pub trait ShaderFile {
    fn file_type(&self) -> &RefCell<FileType>;
    fn content(&self) -> &RefCell<String>;
    fn cache(&self) -> &RefCell<Option<CompileCache>>;
    fn tree(&self) -> &RefCell<Tree>;
    fn line_mapping(&self) -> &RefCell<Vec<usize>>;
    fn include_links(&self) -> Vec<DocumentLink>;

    fn update_from_disc(&self, parser: &mut Parser, file_path: &Path) -> bool {
        if let Ok(content) = read_to_string(file_path) {
            *self.tree().borrow_mut() = parser.parse(&content, None).unwrap();
            *self.line_mapping().borrow_mut() = generate_line_mapping(&content);
            *self.content().borrow_mut() = content;
            true
        } else {
            error!("Unable to read file {}", file_path.to_str().unwrap());
            false
        }
    }

    fn apply_edit(&self, mut changes: Vec<TextDocumentContentChangeEvent>, parser: &mut Parser) {
        let mut content = self.content().borrow_mut();
        let mut tree = self.tree().borrow_mut();
        let mut line_mapping = self.line_mapping().borrow_mut();

        let mut start_index = 0;
        let mut new_content = String::new();

        unsafe {
            changes.sort_by(|a, b| a.range.unwrap().start.cmp(&b.range.unwrap().start));
            changes.iter().for_each(|change| {
                let range = change.range.unwrap();

                let start_byte = byte_index(&content, range.start, &line_mapping);
                let end_byte = byte_index(&content, range.end, &line_mapping);

                let last_line = change.text.split('\n').enumerate().last().unwrap();
                let new_end_position = match last_line.0 {
                    0 => Point {
                        row: range.start.line as usize,
                        column: start_byte.1 + change.text.len(),
                    },
                    lines => Point {
                        row: range.start.line as usize + lines,
                        column: last_line.1.len(),
                    },
                };
                tree.edit(&InputEdit {
                    start_byte: start_byte.0,
                    old_end_byte: end_byte.0,
                    new_end_byte: start_byte.0 + change.text.len(),
                    start_position: Point {
                        row: range.start.line as usize,
                        column: start_byte.1,
                    },
                    old_end_position: Point {
                        row: range.end.line as usize,
                        column: end_byte.1,
                    },
                    new_end_position,
                });
                new_content += content.get_unchecked(start_index..start_byte.0);
                new_content += &change.text;
                start_index = end_byte.0;
            });
            new_content += content.get_unchecked(start_index..);
        }

        *content = new_content;
        *tree = parser.parse(content.as_bytes(), Some(&tree)).unwrap();
        *line_mapping = generate_line_mapping(&content);
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct CompileCache {
    index: u8,
    cache: [u64; 8],
}

#[derive(Clone)]
pub struct WorkspaceFile {
    /// Type of the shader
    file_type: RefCell<FileType>,
    /// The shader pack path that this file in
    shader_pack: Rc<ShaderPack>,
    /// Live content for this file
    content: RefCell<String>,
    /// Range of `#version` macro line. None if this file does not contain `#version`
    version: RefCell<Option<(usize, usize)>>,
    /// Cache to store previously shader code that passed compile
    ///
    /// Only available for shader files
    cache: RefCell<Option<CompileCache>>,
    /// Live syntax tree for this file
    tree: RefCell<Tree>,
    /// Line-content mapping
    line_mapping: RefCell<Vec<usize>>,
    /// Lines that should ignore when merging files and their comment types at the end.
    ///
    /// Currently only contains `#line` and `#version` macro
    ignored_lines: RefCell<Vec<(usize, CommentType)>>,
    /// Files that directly include this file
    included_files: RefCell<HashMap<Rc<PathBuf>, Rc<Self>>>,
    /// Lines and paths for include files
    including_files: RefCell<Vec<IncludeInformation>>,
    /// Shaders Files that include this file, and diagnostics related to them
    parent_shaders: RefCell<HashMap<Rc<PathBuf>, ShaderData>>,
}

#[derive(Clone)]
pub struct TempFile {
    /// Type of the shader
    file_type: RefCell<FileType>,
    /// The shader pack path that this file in
    shader_pack: ShaderPack,
    /// Live content for this file
    content: RefCell<String>,
    /// Range of `#version` macro line. None if this file does not contain `#version`
    version: RefCell<Option<(usize, usize)>>,
    /// Cache to store previously shader code that passed compile
    ///
    /// Only available for shader files
    cache: RefCell<Option<CompileCache>>,
    /// Live syntax tree for this file
    tree: RefCell<Tree>,
    /// Line-content mapping
    line_mapping: RefCell<Vec<usize>>,
    /// Lines that should ignore when merging files and their comment types at the end.
    ///
    /// Currently only contains `#line` and `#version` macro
    ignored_lines: RefCell<Vec<(usize, CommentType)>>,
    /// Lines and paths for include files
    including_files: RefCell<Vec<(usize, usize, usize, PathBuf)>>,
}

#[derive(Clone)]
pub struct ShaderPack {
    pub path: PathBuf,
}

impl core::hash::Hash for ShaderPack {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.path.hash(state);
    }
}

impl PartialEq for ShaderPack {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

impl Eq for ShaderPack {}
