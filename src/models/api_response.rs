use serde::Serialize;

#[derive(Serialize)]
pub struct ApiResponse {
    pub status: String,
    pub message: String,
}
