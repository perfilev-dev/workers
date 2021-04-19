use sha2::{Sha256, Sha512, Digest};
use std::str::FromStr;
use serde::{Serialize, Deserialize};
use std::mem::swap;
use reqwest::StatusCode;


#[derive(Serialize, Deserialize)]
struct Challenge {
    pub bytes: String,
    pub nonce: i32
}

impl Challenge {
    pub fn check(&self, solution: i32) -> bool {

        let mut hasher = Sha256::new();
        hasher.update(&self.bytes);
        hasher.update(i32::to_be_bytes(solution));
        let result = hasher.finalize();
        result.ends_with(&i8::to_be_bytes(self.nonce as i8))
        //result.ends_with(&i8::to_be_bytes(self.nonce))

    }

    pub fn find_solution(&self) -> i32 {

        let mut i = 0;
        loop {
            if self.check(i) {
                return i;
            }
            i += 1;
        }

    }
}

#[derive(Serialize, Deserialize)]
struct ChallengeResponse {
    status: String,
    challenge: Challenge
}

#[derive(Serialize, Deserialize)]
struct RegisterRequest {
    challenge: Challenge,
    solution: i32
}


#[derive(Serialize, Deserialize)]
struct RegisterResponse {
    pub status: String,
    pub token: String
}

fn main() {

    let ip_address = "http://localhost:8000";

    let challenge_response = get_challenge(ip_address).unwrap();
    let find_solution = challenge_response.challenge.find_solution();

    let register_result = register(ip_address, challenge_response, find_solution).unwrap();

    let a = 3;

    return;
}

fn get_challenge(ip_address: &str) -> Result<ChallengeResponse, reqwest::Error> {

    let client = reqwest::blocking::Client::new();
    let resp = client.get(format!("{}/challenge", ip_address))
        .send();

    resp?.json()
}

fn register(ip_address: &str, challenge_response: ChallengeResponse, solution: i32) -> Result<RegisterResponse, reqwest::Error> {

    let client = reqwest::blocking::Client::new();

    let register_request = RegisterRequest {
        challenge: challenge_response.challenge,
        solution
    };

    let resp = client.post( format!("{}/register", ip_address))
        .json(&register_request)
        .send();

    let a = resp.unwrap().text().unwrap();

    let t = 0;

    panic!("aaa");
    //resp?.json();
}



