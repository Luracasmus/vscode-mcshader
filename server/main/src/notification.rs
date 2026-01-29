use serde::{Deserialize, Serialize};
use tower_lsp::lsp_types::notification::Notification;

#[derive(Clone, Deserialize, Serialize)]
pub struct StatusUpdateParams {
    pub status: String,
    pub message: String,
    pub icon: String,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum StatusUpdate {}

impl Notification for StatusUpdate {
    type Params = StatusUpdateParams;

    const METHOD: &'static str = "mcshader/status";
}
