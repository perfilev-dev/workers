#[macro_use]
extern crate lazy_static;

use std::time::{Duration, SystemTime};

use shared::api::*;
use shared::error::*;
use std::ops::Add;
use std::thread::sleep;

lazy_static! {

    // how often update will be checked
    static ref CHECK_UPDATE_TIMEOUT: Duration = {
        let secs = if cfg!(debug_assertions) {
            60
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

}

fn check_update(api: &mut Api) -> Result<()> {
    println!("checking update...");

    Ok(())
}

fn send_heartbeat(api: &mut Api) -> Result<()> {
    println!("sending heartbeat...");

    Ok(())
}

fn main_loop(api: &mut Api) -> Result<()> {
    api.login()?;
    println!("logged in!");

    let mut next_check_update = SystemTime::now().add(CHECK_UPDATE_TIMEOUT.clone());
    let mut next_heartbeat = SystemTime::now();

    loop {
        // checking update...
        if SystemTime::now() > next_check_update {
            next_check_update = SystemTime::now().add(CHECK_UPDATE_TIMEOUT.clone());
            check_update(api);
        }

        // sending heartbeat...
        if SystemTime::now() > next_heartbeat {
            next_heartbeat = SystemTime::now().add(HEARTBEATS_TIMEOUT.clone());
            send_heartbeat(api);
        }

        sleep(Duration::from_secs(1));
    }
}

fn main() {
    let mut api = Api::new("localhost", 8000, false);

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
