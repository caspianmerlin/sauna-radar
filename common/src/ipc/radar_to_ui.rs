use serde::{Serialize, Deserialize};

use crate::api_requests::ApiRequestType;



#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PacketType {
    ApiRequest(ApiRequestType),
}