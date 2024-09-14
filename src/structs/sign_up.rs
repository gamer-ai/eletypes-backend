use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct SignUpRequest {
    pub username: String,
    pub password: String,
    pub confirmation_password: String,
    pub token: String,
}
