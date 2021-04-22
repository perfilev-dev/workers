use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use sha2::digest::Reset;
use sha2::{Digest, Sha256, Sha512};
use shared::{api::*, challenge::Challenge, utils::sha256};
use std::fs;
use std::fs::{read, File};
use std::io::Write;
use std::mem::swap;
use std::str::FromStr;

fn main() {
    let mut api_client = Api::new("localhost", 8000, false);

    let challenge_response = api_client.get_challenge().unwrap();
    let solution = challenge_response.challenge.solve();
    api_client.token = api_client
        .register(challenge_response, solution)
        .unwrap()
        .token;

    let upload = api_client.client_download().unwrap();

    save(upload);

    return;
}

fn save(upload: UploadParameters) {
    let bytes = base64::decode(&upload.base64).unwrap();

    // verify binary against signature?
    let sign_ok =
        shared::utils::verify_sign(&bytes, &upload.sign, &shared::utils::KEY.lock().unwrap())
            .unwrap();

    if !sign_ok {
        return panic!();
    }

    // save binary
    let digest = hex::encode(sha256(&bytes));
    let mut file = File::create(format!("{}.exe", &digest)).unwrap();
    file.write_all(&bytes).unwrap();
}
