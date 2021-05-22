use crate::challenge::Challenge;
use crate::error::*;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Deserialize, Serialize)]
pub struct ChallengeResponse {
    pub challenge: Challenge,
    pub token: String,
}

#[derive(Deserialize, Serialize)]
pub struct ClientInfo {
    pub sha256: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RegisterParameters {
    pub solution: i32,
    pub token: String,
    pub cpu_total: f32,
    pub mem_total: f32,
    pub timestamp: i32
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

#[derive(Deserialize, Serialize)]
pub struct HeartbeatParameters {
    pub token: String, // todo: remove from here!
    pub cpu_usage: f32,
    pub mem_usage: f32,
    pub timestamp: i32
}

#[derive(Deserialize, Serialize)]
pub struct HeartbeatResponse {
    pub success: bool
}

pub struct SystemInfo {
    pub cpu_total: f32,
    pub mem_total: f32,
}

pub struct Api {
    host: String,
    port: usize,
    secure: bool,
    client: Client,
    token: String,
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

    // PUBLIC API

    pub fn token(&mut self) -> String {
        self.token.to_string()
    }

    pub fn login(&mut self, info: &SystemInfo) -> Result<()> {
        loop {
            let challenge_response = self.get_challenge()?;
            if let Some(solution) = challenge_response.challenge.solve() {
                self.token = self.register(challenge_response, solution, info)?.token;
                return Ok(());
            }
        }
    }

    pub fn client_info(&self) -> Result<ClientInfo> {
        Ok(self
            .client
            .get(&self.url("/w/client/info"))
            .header("token", &self.token)
            .send()?
            .json()?)
    }

    pub fn client_download(&self) -> Result<UploadParameters> {
        Ok(self
            .client
            .get(&self.url("/w/client/download"))
            .header("token", &self.token)
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

    pub fn send_heartbeat(&self, parameters: HeartbeatParameters) -> Result<HeartbeatResponse> {
        Ok(self
            .client
            .post(&self.url("/w/heartbeat"))
            .header("token", &self.token)
            .json(&parameters)
            .send()?
            .json()?)
    }

    // PRIVATE API

    fn get_challenge(&self) -> Result<ChallengeResponse> {
        Ok(self.client.get(&self.url("/challenge")).send()?.json()?)
    }

    fn register(
        &self,
        challenge_response: ChallengeResponse,
        solution: i32,
        info: &SystemInfo
    ) -> Result<RegisterResponse> {
        let register_request = RegisterParameters {
            token: challenge_response.token,
            cpu_total: info.cpu_total,
            mem_total: info.mem_total,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i32,
            solution,
        };

        Ok(self
            .client
            .post(&self.url("/register"))
            .json(&register_request)
            .send()?
            .json()?)
    }

    // UTILS

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
