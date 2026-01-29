use glslang::error::GlslangError;
use glslang::{Compiler, CompilerOptions, Program, Shader, ShaderInput, ShaderStage};
use glslang::{OpenGlVersion, ShaderMessage, ShaderOptions, ShaderSource, Target};

fn compile_shader(source: &str, stage: ShaderStage) -> Result<(), GlslangError> {
    let compiler = Compiler::acquire().unwrap();

    let source = ShaderSource::from(source);
    // let limits = ResourceLimits::default(); // TODO: Match Iris.
    let input = ShaderInput::new(
        &source,
        stage,
        &CompilerOptions {
            target: Target::OpenGL {
                version: OpenGlVersion::OpenGL4_5,
                spirv_version: None,
            },
            //version_profile: Some((460, GlslProfile::None)), // TODO: Just override lower versions up to 330.
            messages: ShaderMessage::DEFAULT | ShaderMessage::ENHANCED | ShaderMessage::CASCADING_ERRORS, // COLUMN gives lineoffset
            // See: https://github.com/KhronosGroup/glslang/blob/main/glslang/Public/ShaderLang.h#L255
            ..Default::default()
        },
        Some(&[("mc_chunkFade", Some("(1.0)"))]),
        //None,
        None,
    )?;

    let mut shader = Shader::new(compiler, input)?;
    shader.options(ShaderOptions::AUTO_MAP_BINDINGS | ShaderOptions::AUTO_MAP_LOCATIONS); // ShaderOptions::VULKAN_RULES_RELAXED

    let mut program = Program::new(compiler);

    program.add_shader(&shader);

    program.link()?;

    Ok(())
}

pub fn validate_shader(shader_stage: ShaderStage, source: &str) -> Option<GlslangError> {
    // Patch away `uniform` qualifiers from non-opaque variables to make glslang happy.
    const UNIFORM_QUALIFIER: &str = "uniform";
    let source: Box<str> = source
        .split_inclusive(';')
        .map(|statement| {
            if let Some(uniform_qualifier) = statement.rfind(UNIFORM_QUALIFIER) {
                let uniform_qualifier_end = uniform_qualifier + UNIFORM_QUALIFIER.len();

                let after_uniform_qualifier = &statement[uniform_qualifier_end..(statement.len() - 1)];

                if !(after_uniform_qualifier.contains("image") ^ after_uniform_qualifier.contains("sampler")) {
                    format!("{}{}", &statement[..uniform_qualifier], &statement[uniform_qualifier_end..])
                } else {
                    statement.to_owned()
                }
            } else {
                statement.to_owned()
            }
            .replace("#warning ", "#error ")
        })
        .collect();

    /*const UNIFORM_QUALIFIER: &str = "uniform";
    source
        .match_indices(UNIFORM_QUALIFIER)
        .fold(0, |location, (uniform, _)| {
            if let Some(semicolon) = source[uniform..].find(';') {
                let uniform_str = &source[(uniform + UNIFORM_QUALIFIER.len())..semicolon];

                let last_closing_paren = uniform_str.rfind(')').unwrap_or(0);
            } else {
                0 // TODO: Error.
            }

            /*if let Some(uniform) = line.find("uniform")
                && !(line.contains(" sampler") ^ line.contains(" image"))
            // TODO: Check this better.
            {
                let new_line = format!("layout(location = {accum}) {line}\n");
                accum += 1;
                new_line
            } else {
                format!("{line}\n")
            }*/

            /*if let Some(uniform) = line.find(UNIFORM_QUALIFIER)
            // TODO: Check this better.
            {
                let last_closing_paren = line.rfind(')').unwrap_or(0);

                let splits: Box<[usize]> = line
                    .match_indices(',')
                    .filter_map(|(i, _)| if i > last_closing_paren { Some(i) } else { None })
                    .collect();
                let first_variable = if let Some(first_split) = splits.first() {
                    line[uniform..first_split]
                } else {
                    line[uniform..]
                }.rfind(' ')

                let mut new_line = line.to_owned();
                new_line.insert_str(uniform + UNIFORM_QUALIFIER.len(), format!("layout(location = {accum}) ").as_str());

                accum += 1;

                new_line.push('\n');

                new_line
            } else {
                new_source.push_str(line);
                new_source.push('\n');
                0
            }*/
        })
        .collect();*/

    compile_shader(&source, shader_stage).err()

    /*unsafe {
        let shader = gl::CreateShader(file_type);
        let c_str_frag = CString::new(source).unwrap();
        gl::ShaderSource(shader, 1, &c_str_frag.as_ptr(), ptr::null());
        gl::CompileShader(shader);

        // Check for shader compilation errors
        let mut success = gl::FALSE as i32;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);
        let result = if success == gl::TRUE as i32 {
            None
        } else {
            let mut info_len: c_int = 0;
            gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut info_len);
            let mut info = Vec::with_capacity(info_len as usize);
            gl::GetShaderInfoLog(shader, info_len, ptr::null_mut(), info.as_mut_ptr() as *mut gl::types::GLchar);

            // ignore null for str::from_utf8
            let info_len = match info_len {
                0 => 0,
                _ => (info_len - 1) as usize,
            };
            info.set_len(info_len);
            Some(String::from_utf8_unchecked(info))
        };
        gl::DeleteShader(shader);
        result
    }*/
}
