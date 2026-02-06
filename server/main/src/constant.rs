use std::sync::LazyLock;

use hashbrown::{HashMap, HashSet};
use regex::Regex;

use crate::commands::*;

/// In GLSL 1.50 and later, the offset is 1. We're forced to use at least 3.30+ in shaderc, so we don't need to detect this at runtime.
pub const LINE_OFFSET: u32 = 1;

pub static BASIC_EXTENSIONS: LazyLock<HashSet<Box<str>>> = LazyLock::new(|| {
    HashSet::from([
        Box::from("csh"),
        Box::from("vsh"),
        Box::from("gsh"),
        Box::from("fsh"),
        Box::from("tcs"),
        Box::from("tes"),
        Box::from("glsl"),
    ])
});
pub static RE_BASIC_SHADERS: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
    r"^(dh_(terrain|water|shadow)|shadow(|_solid|_cutout|_water|_entities|_block)|gbuffers_(armor_glint|basic|beaconbeam|block|clouds|damagedblock|entities|entities_glowing|hand|hand_water|line|skybasic|skytextured|spidereyes|terrain|textured|textured_lit|water|weather|particles|particles_translucent|block_translucent|entities_translucent|terrain_solid|terrain_cutout|lightning)).(vsh|gsh|fsh|tcs|tes)|setup([1-9]\d?)?.csh|(final|(begin|shadowcomp|prepare|deferred|composite)([1-9]\d?)?)(.vsh|.gsh|.fsh|(_[a-z])?.csh)$"
).unwrap()
});
pub static COMMAND_LIST: LazyLock<HashMap<&'static str, Box<dyn Command + Sync + Send>>> =
    LazyLock::new(|| HashMap::from([("virtualMerge", Box::new(VirtualMerge {}) as Box<dyn Command + Sync + Send>)]));
pub static RE_DIMENSION_FOLDER: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^world-?\d+$").unwrap());
// pub static RE_MACRO_PARSER_MULTI_LINE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"(?m)^[ \f\t\v]*#\s*((include|moj_import)\s+[<"](.+)[>"]|line|version).?$"#).unwrap());
pub static RE_MACRO_PARSER: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"^\s*#\s*(include\s+"(.+)"|line|version)"#).unwrap());
pub static RE_MACRO_PARSER_TEMP: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"^\s*#\s*((include|moj_import)\s+[<"](.+)[>"]|line|version)"#).unwrap());
pub static RE_MACRO_VERSION: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[ \f\t\v]*#\s*version[ \f\t\v]+(\d+)([ \f\t\v]+[a-z]+)?").unwrap());
pub static RE_COMMENT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"/[/*]|\*/|\\\r?$").unwrap());
pub static DIAGNOSTICS_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(?P<filepath>\d+)\:(?P<linenum>\d+)\: (?P<severity>error|warning): '(?P<item>.*)' : (?P<output>.+)").unwrap()
});
pub static SHADERC_COMPILER: LazyLock<shaderc::Compiler> = LazyLock::new(|| shaderc::Compiler::new().unwrap());

#[cfg(not(target_os = "linux"))]
#[cfg(not(target_os = "windows"))]
#[cfg(not(target_os = "macos"))]
pub const IRIS_OS_MACRO: &str = "MC_OS_OTHER";

#[cfg(target_os = "linux")]
pub const IRIS_OS_MACRO: &str = "MC_OS_LINUX";

#[cfg(target_os = "windows")]
pub const IRIS_OS_MACRO: &str = "MC_OS_WINDOWS";

#[cfg(target_os = "macos")]
pub const IRIS_OS_MACRO: &str = "MC_OS_MAC";

pub const IRIS_MACROS: [(&str, Option<&str>); 75] = [
    ("IS_LSP_MCSHADER", None),
    ("MC_VERSION", Some("12111")),
    ("IS_IRIS", None),
    ("IRIS_HAS_TRANSLUCENCY_SORTING", None),
    ("IRIS_TAG_SUPPORT", Some("2")),
    ("IRIS_VERSION", Some("11004")),
    ("IRIS_HAS_CONNECTED_TEXTURES", None),
    ("MC_MIPMAP_LEVEL", Some("4")),
    ("MC_GL_VERSION", Some("450")),
    ("MC_GLSL_VERSION", Some("330")),
    ("MC_GL_VENDOR_OTHER", None),
    ("MC_GL_RENDERER_OTHER", None),
    ("MC_NORMAL_MAP", None),
    ("MC_SPECULAR_MAP", None),
    ("MC_RENDER_QUALITY", Some("1.0")),
    ("MC_SHADOW_QUALITY", Some("1.0")),
    ("MC_HAND_DEPTH", Some("0.125")),
    ("MC_RENDER_STAGE_NONE", Some("0")),
    ("MC_RENDER_STAGE_SKY", Some("1")),
    ("MC_RENDER_STAGE_SUNSET", Some("2")),
    ("MC_RENDER_STAGE_SUN", Some("4")),
    ("MC_RENDER_STAGE_CUSTOM_SKY", Some("3")),
    ("MC_RENDER_STAGE_MOON", Some("5")),
    ("MC_RENDER_STAGE_STARS", Some("6")),
    ("MC_RENDER_STAGE_VOID", Some("7")),
    ("MC_RENDER_STAGE_TERRAIN_SOLID", Some("8")),
    ("MC_RENDER_STAGE_TERRAIN_CUTOUT_MIPPED", Some("9")),
    ("MC_RENDER_STAGE_TERRAIN_CUTOUT", Some("10")),
    ("MC_RENDER_STAGE_ENTITIES", Some("11")),
    ("MC_RENDER_STAGE_BLOCK_ENTITIES", Some("12")),
    ("MC_RENDER_STAGE_DESTROY", Some("13")),
    ("MC_RENDER_STAGE_OUTLINE", Some("14")),
    ("MC_RENDER_STAGE_DEBUG", Some("15")),
    ("MC_RENDER_STAGE_HAND_SOLID", Some("16")),
    ("MC_RENDER_STAGE_TERRAIN_TRANSLUCENT", Some("17")),
    ("MC_RENDER_STAGE_TRIPWIRE", Some("18")),
    ("MC_RENDER_STAGE_PARTICLES", Some("19")),
    ("MC_RENDER_STAGE_CLOUDS", Some("20")),
    ("MC_RENDER_STAGE_RAIN_SNOW", Some("21")),
    ("MC_RENDER_STAGE_WORLD_BORDER", Some("22")),
    ("MC_RENDER_STAGE_HAND_TRANSLUCENT", Some("23")),
    ("DH_BLOCK_UNKNOWN", Some("0")),
    ("DH_BLOCK_LEAVES", Some("1")),
    ("DH_BLOCK_STONE", Some("2")),
    ("DH_BLOCK_WOOD", Some("3")),
    ("DH_BLOCK_METAL", Some("4")),
    ("DH_BLOCK_DIRT", Some("5")),
    ("DH_BLOCK_LAVA", Some("6")),
    ("DH_BLOCK_DEEPSLATE", Some("7")),
    ("DH_BLOCK_SNOW", Some("8")),
    ("DH_BLOCK_SAND", Some("9")),
    ("DH_BLOCK_TERRACOTTA", Some("10")),
    ("DH_BLOCK_NETHER_STONE", Some("11")),
    ("DH_BLOCK_WATER", Some("12")),
    ("DH_BLOCK_GRASS", Some("13")),
    ("DH_BLOCK_AIR", Some("14")),
    ("DH_BLOCK_ILLUMINATED", Some("15")),
    ("DISTANT_HORIZONS", None),
    ("mc_chunkFade", Some("(1.0)")),
    // Compatibility profile:
    ("gl_Color", Some("vec4(1.0)")),
    ("gl_Normal", Some("vec3(1.0)")),
    ("gl_Vertex", Some("vec4(1.0)")),
    ("gl_MultiTexCoord0", Some("vec4(1.0)")),
    ("gl_MultiTexCoord1", Some("vec4(1.0)")),
    ("gl_MultiTexCoord2", Some("vec4(1.0)")),
    ("gl_NormalMatrix", Some("mat3(1.0)")),
    ("gl_TextureMatrix", Some("mat4[2](mat4(1.0), mat4(1.0))")),
    ("gl_ModelViewMatrix", Some("mat4(1.0)")),
    ("gl_ModelViewProjectionMatrix", Some("mat4(1.0)")),
    ("gl_ProjectionMatrix", Some("mat4(1.0)")),
    ("ftransform()", Some("vec4(1.0)")),
    // Distant Horizons:
    ("dhMaterialId", Some("(1)")),
    ("gl_ModelViewMatrixInverse", Some("mat4(1.0)")),
    ("gl_ProjectionMatrixInverse", Some("mat4(1.0)")),
    (IRIS_OS_MACRO, None),
];
