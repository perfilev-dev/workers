use sha2::{Sha256, Sha512, Digest};
use std::str::FromStr;
use serde::{Serialize, Deserialize};
use std::mem::swap;
use reqwest::StatusCode;
use shared::{challenge::Challenge, api::*, utils::sha256};
use sha2::digest::Reset;
use std::fs;
use std::fs::{File, read};
use std::io::Write;

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




    pub fn client_download(&self) -> Result<UploadParameters, reqwest::Error> {

        let client = reqwest::blocking::Client::new();

        let resp = client.get( format!("{}/w/client/download", &self.ip_address))
            .header("token", &self.token)
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

    let upload = api_server.client_download().unwrap();

    save(upload);

    return;
}

fn save (upload : UploadParameters) {

    let bytes = base64::decode(&upload.base64).unwrap();

    // verify binary against signature?
    let sign_ok = shared::utils::verify_sign(&bytes, &upload.sign, &shared::utils::KEY.lock().unwrap()).unwrap();

    if !sign_ok {
        return panic!();
    }


    // save binary
    let digest = hex::encode(sha256(&bytes));
    let mut file = File::create(format!("{}.exe", &digest)).unwrap();
    file.write_all(&bytes).unwrap();

}