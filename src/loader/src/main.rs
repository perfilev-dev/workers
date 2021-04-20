use sha2::{Sha256, Sha512, Digest};
use std::str::FromStr;
use serde::{Serialize, Deserialize};
use std::mem::swap;
use reqwest::StatusCode;
use shared::{challenge::Challenge, api::*};

struct ApiServer {

    pub ip_address: String,
    pub token: String
}

impl ApiServer {

    fn new(ip_address: &str) -> ApiServer {
        ApiServer { ip_address: ip_address.to_string(), token: "".to_string() }
    }

    pub fn get_challenge(&self) -> Result<ChallengeResponse, reqwest::Error> {

        let client = reqwest::blocking::Client::new();
        let resp = client.get(format!("{}/challenge", &self.ip_address))
            .send();

        resp?.json()
    }

    pub fn register(&self, challenge_response: ChallengeResponse, solution: i32) -> Result<RegisterResponse, reqwest::Error> {

        let client = reqwest::blocking::Client::new();

        let register_request = RegisterParameters {
            token: challenge_response.token,
            solution
        };

        let resp = client.post( format!("{}/register", self.ip_address))
            .json(&register_request)
            .send();


        resp?.json()
    }


}

fn main() {

    let ip_address = "http://localhost:8000";

    let mut api_server = ApiServer::new(ip_address);
    let challenge_response = api_server.get_challenge().unwrap(); //get_challenge(ip_address).unwrap();
    let solution = challenge_response.challenge.solve();
    api_server.token = api_server.register(challenge_response, solution).unwrap().token;


    let a = 3;

    return;
}

