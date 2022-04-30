use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HttpMethod {
    Post,
    Put,
    Get,
    Patch,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct ApiResponse {
    pub result: String,
}
