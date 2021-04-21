use crate::challenge::Challenge;
use serde::{Serialize, Deserialize};


#[derive(Deserialize, Serialize)]
pub struct ChallengeResponse {
    pub challenge: Challenge,
    pub token: String
}

#[derive(Deserialize, Serialize)]
pub struct ClientInfo {
    pub sha256: String
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

#[derive(Deserialize, Serialize)]
pub struct UploadParameters {
    pub base64: String,
    pub sign: String
}

#[derive(Deserialize, Serialize)]
pub struct UploadResponse {
    pub sha256: String
}
