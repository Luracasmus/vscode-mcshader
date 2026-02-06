use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
    rc::Rc,
};

use shaderc::{
    CompileOptions, EnvVersion, IncludeCallbackResult, IncludeType, Limit, OptimizationLevel, ResolvedInclude, ShaderKind, SpirvVersion,
    TargetEnv,
};

use crate::constant::{IRIS_MACROS, SHADERC_COMPILER};

#[must_use]
pub fn validate_shader(shader_kind: ShaderKind, source: &str, pack_path: &Path, shader_path: &Rc<PathBuf>) -> Option<shaderc::Error> {
    let mut options = CompileOptions::new().unwrap();
    options.set_auto_bind_uniforms(true);
    options.set_auto_map_locations(true);
    options.set_target_env(TargetEnv::OpenGL, EnvVersion::OpenGL4_5 as u32);
    options.set_target_spirv(SpirvVersion::V1_6);
    options.set_warnings_as_errors();
    options.set_optimization_level(OptimizationLevel::Zero);
    options.set_limit(Limit::MaxAtomicCounterBindings, 0); // Not supported in Iris.
    options.set_forced_version_profile(460, shaderc::GlslProfile::Core);
    options.set_include_callback(|included_name, inc_type, including_resolved_name, _probablydepthidk| {
        if inc_type == IncludeType::Standard {
            let (dir_path, rel_path) = if included_name.starts_with('/') {
                (pack_path, included_name.trim_start_matches('/'))
            } else {
                (Path::new(including_resolved_name).parent().unwrap(), included_name)
            };

            let mut resolved_path = dir_path.to_owned();
            resolved_path.push(rel_path);

            let mut file = File::open(resolved_path.clone()).map_err(|err| err.to_string())?;
            let mut content = String::new();

            file.read_to_string(&mut content).map_err(|err| err.to_string())?;

            IncludeCallbackResult::Ok(ResolvedInclude {
                resolved_name: resolved_path.to_string_lossy().to_string(),
                content,
            })
        } else {
            IncludeCallbackResult::Err("Shaderc relative-style include directives are unsupported".to_owned())
        }
    });

    IRIS_MACROS.into_iter().for_each(|iris_macro| {
        options.add_macro_definition(iris_macro.0, iris_macro.1);
    });
    // TODO: options.set_limit(Limit, value); // To match Iris.

    /*SHADERC_COMPILER
    .preprocess(
        source.replace("#warning ", "#error ").as_str(),
        //shader_kind,
        shader_path.to_str().unwrap_or_default(),
        "main",
        Some(&options),
    )
    .map(|spv| logging::info!("{}", spv.as_text()))
    .err()?;*/

    SHADERC_COMPILER
        .compile_into_spirv(
            source.replace("#warning ", "#error ").as_str(),
            shader_kind,
            shader_path.to_str().unwrap_or_default(),
            "main",
            Some(&options),
        )
        .err()
}
