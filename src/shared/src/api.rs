use crate::challenge::Challenge;
use crate::error::*;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct ChallengeResponse {
    pub challenge: Challenge,
    pub token: String,
}

#[derive(Deserialize, Serialize)]
pub struct ClientInfo {
    pub sha256: String,
}

#[derive(Deserialize, Serialize)]
pub struct RegisterParameters {
    pub solution: i32,
    pub token: String,
}

#[derive(Deserialize, Serialize)]
pub struct RegisterResponse {
    pub token: String,
}

#[derive(Deserialize, Serialize)]
pub struct UploadParameters {
    pub base64: String,
    pub sign: String,
}

#[derive(Deserialize, Serialize)]
pub struct UploadResponse {
    pub sha256: String,
}

pub struct Api {
    host: String,
    port: usize,
    secure: bool,
    client: Client,
    pub token: String,
}

impl Api {
    pub fn new(host: &str, port: usize, secure: bool) -> Api {
        Api {
            host: host.to_string(),
            port,
            secure,
            client: Client::new(),
            token: "".to_string(),
        }
    }

    pub fn client_download(&self) -> Result<UploadParameters> {
        Ok(self
            .client
            .get(&self.url("/w/client/download"))
            .header("token", &self.token)
            .send()?
            .json()?)
    }

    pub fn get_challenge(&self) -> Result<ChallengeResponse> {
        Ok(self.client.get(&self.url("/challenge")).send()?.json()?)
    }

    pub fn register(
        &self,
        challenge_response: ChallengeResponse,
        solution: i32,
    ) -> Result<RegisterResponse> {
        let register_request = RegisterParameters {
            token: challenge_response.token,
            solution,
        };

        Ok(self
            .client
            .post(&self.url("/register"))
            .json(&register_request)
            .send()?
            .json()?)
    }

    pub fn upload_binary(&self, parameters: UploadParameters) -> Result<UploadResponse> {
        Ok(self
            .client
            .post(&self.url("/c/binary"))
            .json(&parameters)
            .send()?
            .json()?)
    }

    fn url(&self, method: &str) -> String {
        format!(
            "http{}://{}:{}{}",
            if self.secure { "s" } else { "" },
            self.host,
            self.port,
            method
        )
    }
}
