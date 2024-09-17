use serde::Serialize;

#[derive(Serialize)]
pub struct ApiResponse {
    pub status: String,
    pub message: String,
}

#[derive(Serialize)]
pub struct ApiResponseWithData<T> {
    pub status: String,
    pub message: String,
    pub data: T,
}

pub fn create_api_response(status: &str, message: &str) -> ApiResponse {
    ApiResponse {
        status: status.to_string(),
        message: message.to_string(),
    }
}

pub fn create_api_response_with_data<T>(
    status: &str,
    message: &str,
    data: T,
) -> ApiResponseWithData<T> {
    ApiResponseWithData {
        status: status.to_string(),
        message: message.to_string(),
        data,
    }
}

pub fn success_response(message: &str) -> ApiResponse {
    create_api_response("success", message)
}

pub fn error_response(message: &str) -> ApiResponse {
    create_api_response("error", message)
}

pub fn success_response_with_data<T>(message: &str, data: T) -> ApiResponseWithData<T> {
    create_api_response_with_data("success", message, data)
}

pub fn error_response_with_data<T>(message: &str, data: T) -> ApiResponseWithData<T> {
    create_api_response_with_data("error", message, data)
}
