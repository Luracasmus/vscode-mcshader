use std::sync::LazyLock;

use hashbrown::{HashMap, HashSet};
use regex::Regex;

use crate::commands::*;
use crate::opengl::OpenGlContext;

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
pub static OPENGL_CONTEXT: LazyLock<OpenGlContext> = LazyLock::new(OpenGlContext::new);
pub static DIAGNOSTICS_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    match OPENGL_CONTEXT.vendor() {
        "NVIDIA Corporation" => {
            Regex::new(r"^(?P<filepath>\d+)\((?P<linenum>\d+)\) : (?P<severity>error|warning) [A-C]\d+: (?P<output>.+)")
        }
        #[cfg(target_os = "linux")]
        "AMD" => Regex::new(
            // We assume RadeonSI.
            r"^(?P<filepath>\d+)\:(?P<linenum>\d+)\((?P<lineoffset>\d+)\): (?P<severity>error|warning): (?P<output>.+)",
        ),
        _ => Regex::new(
            r#"^(?P<severity>ERROR|WARNING): (?P<filepath>[^?<>*|"\n]+):(?P<linenum>\d+): (?:'.*' :|[a-z]+\(#\d+\)) +(?P<output>.+)$"#,
        ),
    }
    .unwrap()
});
pub static IRIS_MACROS: LazyLock<Box<str>> = LazyLock::new(|| {
    let mut iris_macros = String::from(
        "#define IS_LSP_MCSHADER
    #define MC_VERSION 12111
    #define IS_IRIS
    #define IRIS_HAS_TRANSLUCENCY_SORTING
    #define IRIS_TAG_SUPPORT 2
    #define IRIS_VERSION 11004
    #define IRIS_HAS_CONNECTED_TEXTURES
    #define MC_MIPMAP_LEVEL 4
    #define MC_GL_VERSION 320
    #define MC_GLSL_VERSION 150
    #define MC_NORMAL_MAP
    #define MC_SPECULAR_MAP
    #define MC_RENDER_QUALITY 1.0
    #define MC_SHADOW_QUALITY 1.0
    #define MC_HAND_DEPTH 0.125
    #define MC_RENDER_STAGE_NONE 0
    #define MC_RENDER_STAGE_SKY 1
    #define MC_RENDER_STAGE_SUNSET 2
    #define MC_RENDER_STAGE_SUN 4
    #define MC_RENDER_STAGE_CUSTOM_SKY 3
    #define MC_RENDER_STAGE_MOON 5
    #define MC_RENDER_STAGE_STARS 6
    #define MC_RENDER_STAGE_VOID 7
    #define MC_RENDER_STAGE_TERRAIN_SOLID 8
    #define MC_RENDER_STAGE_TERRAIN_CUTOUT_MIPPED 9
    #define MC_RENDER_STAGE_TERRAIN_CUTOUT 10
    #define MC_RENDER_STAGE_ENTITIES 11
    #define MC_RENDER_STAGE_BLOCK_ENTITIES 12
    #define MC_RENDER_STAGE_DESTROY 13
    #define MC_RENDER_STAGE_OUTLINE 14
    #define MC_RENDER_STAGE_DEBUG 15
    #define MC_RENDER_STAGE_HAND_SOLID 16
    #define MC_RENDER_STAGE_TERRAIN_TRANSLUCENT 17
    #define MC_RENDER_STAGE_TRIPWIRE 18
    #define MC_RENDER_STAGE_PARTICLES 19
    #define MC_RENDER_STAGE_CLOUDS 20
    #define MC_RENDER_STAGE_RAIN_SNOW 21
    #define MC_RENDER_STAGE_WORLD_BORDER 22
    #define MC_RENDER_STAGE_HAND_TRANSLUCENT 23
    #define DH_BLOCK_UNKNOWN 0
    #define DH_BLOCK_LEAVES 1
    #define DH_BLOCK_STONE 2
    #define DH_BLOCK_WOOD 3
    #define DH_BLOCK_METAL 4
    #define DH_BLOCK_DIRT 5
    #define DH_BLOCK_LAVA 6
    #define DH_BLOCK_DEEPSLATE 7
    #define DH_BLOCK_SNOW 8
    #define DH_BLOCK_SAND 9
    #define DH_BLOCK_TERRACOTTA 10
    #define DH_BLOCK_NETHER_STONE 11
    #define DH_BLOCK_WATER 12
    #define DH_BLOCK_GRASS 13
    #define DH_BLOCK_AIR 14
    #define DH_BLOCK_ILLUMINATED 15
    #define DISTANT_HORIZONS\n",
    );

    #[cfg(target_os = "linux")]
    pub const IRIS_OS_MACRO: &str = "#define MC_OS_LINUX\n";

    #[cfg(target_os = "windows")]
    pub const IRIS_OS_MACRO: &str = "#define MC_OS_WINDOWS\n";

    #[cfg(target_os = "macos")]
    pub const IRIS_OS_MACRO: &str = "#define MC_OS_MAC\n";

    iris_macros += IRIS_OS_MACRO;

    let vendor = OPENGL_CONTEXT.vendor().to_ascii_lowercase();

    // Match Iris behavior.
    iris_macros += if vendor.starts_with("ati") {
        "#define MC_GL_VENDOR_ATI\n"
    } else if vendor.starts_with("intel") {
        "#define MC_GL_VENDOR_INTEL\n"
    } else if vendor.starts_with("nvidia") {
        "#define MC_GL_VENDOR_NVIDIA\n"
    } else if vendor.starts_with("amd") {
        "#define MC_GL_VENDOR_AMD\n"
    } else if vendor.starts_with("x.org") {
        "#define MC_GL_VENDOR_XORG\n"
    } else {
        "#define MC_GL_VENDOR_OTHER\n"
    };

    let renderer = OPENGL_CONTEXT.renderer().to_ascii_lowercase();

    // Match Iris behavior.
    iris_macros += if renderer.starts_with("amd") || renderer.starts_with("ati") || renderer.starts_with("radeon") {
        "#define MC_GL_RENDERER_RADEON\n"
    } else if renderer.starts_with("gallium") {
        "#define MC_GL_RENDERER_GALLIUM\n"
    } else if renderer.starts_with("intel") {
        "#define MC_GL_RENDERER_INTEL\n"
    } else if renderer.starts_with("geforce") || renderer.starts_with("nvidia") {
        "#define MC_GL_RENDERER_GEFORCE\n"
    } else if renderer.starts_with("quadro") || renderer.starts_with("nvs") {
        "#define MC_GL_RENDERER_QUADRO\n"
    } else if renderer.starts_with("mesa") {
        "#define MC_GL_RENDERER_MESA\n"
    } else if renderer.starts_with("apple") {
        "#define MC_GL_RENDERER_APPLE\n"
    } else {
        "#define MC_GL_RENDERER_OTHER\n"
    };

    iris_macros.into_boxed_str()
});
