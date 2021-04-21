use serde::{Serialize, Deserialize};
use crate::challenge::Challenge;
use crate::error::*;
use reqwest::blocking::Client;


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

pub struct Api {
    host: String,
    port: usize,
    secure: bool,
    client: Client
}

impl Api {

    pub fn new(host: &str, port: usize, secure: bool) -> Api {
        Api { host: host.to_string(), port, secure, client: Client::new() }
    }

    pub fn upload_binary(&self, parameters: UploadParameters) -> Result<UploadResponse> {
        Ok(self.client.post(&self.url("/c/binary"))
            .json(&parameters)
            .send()?
            .json()?)
    }

    fn url(&self, method: &str) -> String {
        format!("http{}://{}:{}{}", if self.secure {"s"} else {""}, self.host, self.port, method)
    }

}