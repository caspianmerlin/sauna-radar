use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "camelCase")]
pub struct TextCommandRequest {
    pub callsign: String,
    pub command: String,
    pub args: Vec<String>,
}
