use std::borrow::Cow;

use tower_lsp::jsonrpc::{Error, ErrorCode};

use super::LanguageServerError;

impl LanguageServerError {
    #[inline]
    pub const fn not_shader_error() -> Error {
        Error {
            code: ErrorCode::ServerError(-20002),
            message: Cow::Borrowed("This is not a base shader file"),
            data: None,
        }
    }

    #[inline]
    pub const fn invalid_command_error() -> Error {
        Error {
            code: ErrorCode::ServerError(-20101),
            message: Cow::Borrowed("Invalid command"),
            data: None,
        }
    }

    #[inline]
    pub const fn invalid_argument_error() -> Error {
        Error {
            code: ErrorCode::ServerError(-20102),
            message: Cow::Borrowed("Invalid command argument"),
            data: None,
        }
    }
}
