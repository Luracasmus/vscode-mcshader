use shaderc::{CompileOptions, EnvVersion, Limit, OptimizationLevel, ShaderKind, SpirvVersion, TargetEnv};

use crate::constant::{IRIS_MACROS, SHADERC_COMPILER};

#[must_use]
pub fn validate_shader(shader_kind: ShaderKind, source: &str) -> Option<shaderc::Error> {
    let mut options = CompileOptions::new().unwrap();
    options.set_auto_bind_uniforms(true);
    options.set_auto_map_locations(true);
    options.set_target_env(TargetEnv::OpenGL, EnvVersion::OpenGL4_5 as u32);
    options.set_target_spirv(SpirvVersion::V1_6);
    options.set_warnings_as_errors();
    options.set_optimization_level(OptimizationLevel::Zero);
    options.set_limit(Limit::MaxAtomicCounterBindings, 0); // Not supported in Iris.

    IRIS_MACROS.into_iter().for_each(|iris_macro| {
        options.add_macro_definition(iris_macro.0, iris_macro.1);
    });
    // TODO: options.set_limit(Limit, value); // To match Iris.

    SHADERC_COMPILER
        .compile_into_spirv(
            source.replace("#warning ", "#error ").as_str(),
            shader_kind,
            "placeholder file name",
            "main",
            Some(&options),
        )
        .err()

    /* {
        Ok(artifact) => {
            if artifact.get_num_warnings() > 0 {
                // TODO: artifact.get_warning_messages(), unless options.set_warnings_as_errors(); solves this
                None
            } else {
                None
            }
        }
        Err(err) => Some(err),
    }*/
}
