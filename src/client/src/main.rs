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

fn check_update(api: &mut Api) -> Result<bool> {
    println!("checking update...");

    let info = api.client_info()?;
    if info.sha256 != OWN_SHA256.as_ref() {
        println!("downloading update...");

        let binary = api.client_download()?;
        let bytes = base64::decode(&binary.base64).unwrap();

        // verify binary against signature?
        let sign_ok =
            shared::utils::verify_sign(&bytes, &binary.sign, &shared::utils::KEY.lock().unwrap())
                .unwrap();

        if !sign_ok {
            println!("signature is wrong! skipping...");
            return Err("wrong signature!".into());
        }

        // todo: save, run and return Ok()

        println!("successful updated!");
        return Ok(true); // successful update, need to terminate!
    }

    Ok(false) // no update
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

fn main_loop(api: &mut Api) -> Result<()> {
    let sys = systemstat::System::new();
    let info = SystemInfo {
        cpu_total: 0.0, // todo
        mem_total: sys.memory().map(|x| x.total.as_u64()).unwrap_or(0) as f32
    };

    api.login(&info)?;
    println!("logged in!");

    let mut next_check_update = SystemTime::now().add(CHECK_UPDATE_TIMEOUT.clone());
    let mut next_heartbeat = SystemTime::now();

    loop {
        // checking update...
        if SystemTime::now() > next_check_update {
            next_check_update = SystemTime::now().add(CHECK_UPDATE_TIMEOUT.clone());
            let updated = check_update(api)?;
            if updated {
                println!("terminating...");
                return Ok(());
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

fn main() {
    let mut api = Api::new("10.211.55.2", 8000, false);

    loop {
        match main_loop(&mut api) {
            Ok(_) => {
                return;
            }
            Err(err) => {
                println!("error: {}, restarting...", err.to_string());
                sleep(Duration::from_secs(5));
            }
        }
    }
}
