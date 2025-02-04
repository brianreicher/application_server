use serde_derive::{Deserialize, Serialize};

use super::errors;

#[derive(Serialize, Deserialize)]
pub struct RegisterResponse {
    pub token: String,
    pub challenge: Vec<String>,
}
#[derive(Serialize, Deserialize)]
pub struct RegisterRequest {
    pub name: String,
    pub nuid: String,
}

#[derive(Serialize, Deserialize)]
pub struct HandleForgotTokenResponse {
    pub token: String,
}
#[derive(Serialize, Deserialize)]

pub struct GetChallenge {
    pub challenge: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct ErrorResponse<'a> {
    pub msg: &'a str,
    #[serde(flatten)]
    pub error: Option<errors::ApiError>,
}
