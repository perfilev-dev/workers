use crate::challenge::Challenge;
use serde::{Serialize, Deserialize};

#[derive(Deserialize, Serialize)]
pub struct ErrorResponse(pub String);

#[derive(Deserialize, Serialize)]
pub struct ChallengeResponse {
    pub challenge: Challenge,
    pub token: String
}

#[derive(Deserialize, Serialize)]
pub struct RegisterParameters {
    pub solution: i32,
    pub token: String,
}

#[derive(Deserialize, Serialize)]
pub struct RegisterResponse {
    pub token: String
}
