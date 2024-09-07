use serde::Serialize;

#[derive(Serialize)]
pub struct ApiResponse {
    pub status: String,
    pub message: String,
}

pub fn create_api_response(status: &str, message: &str) -> ApiResponse {
    ApiResponse {
        status: status.to_string(),
        message: message.to_string(),
    }
}

pub fn success_response(message: &str) -> ApiResponse {
    create_api_response("success", message)
}

pub fn error_response(message: &str) -> ApiResponse {
    create_api_response("error", message)
}
