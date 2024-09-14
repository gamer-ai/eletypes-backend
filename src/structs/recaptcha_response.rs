use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct RecaptchaResponse {
    pub success: bool,
    pub error_codes: Option<Vec<String>>,
}
