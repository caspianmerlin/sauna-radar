use serde::{Serialize, Deserialize};

use self::text_command::TextCommandRequest;

pub mod text_command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ApiRequestType {
    TextCommand(TextCommandRequest),
}