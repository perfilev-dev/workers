#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate diesel;
#[macro_use] extern crate diesel_migrations;
#[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_contrib;

mod challenge;
mod heartbeat;
mod schema;

use rocket_contrib::json::{Json, JsonValue};
use heartbeat::Heartbeat;
use challenge::Challenge;
use std::net::SocketAddr;

#[database("sqlite_database")]
pub struct DbConn(diesel::SqliteConnection);


#[get("/challenge")]
fn get_challenge(remote_addr: SocketAddr, con: DbConn) -> JsonValue {
    let ip = remote_addr.ip().to_string();

    if Challenge::count_by_ip(&ip, &con).unwrap() > 1000 {
        Challenge::remove_first_with_ip(&ip, &con);
    }

    let challenge = Challenge::new(&remote_addr.ip().to_string());

    let challenges = Challenge::all(&con).unwrap();

    json!({"status": "ok", "challenge": challenge, "challenges": challenges})
}


#[get("/heartbeats")]
fn heartbeats(con: DbConn) -> JsonValue {
    match Heartbeat::all(con) {
        Ok(heartbeats) => json!({"status": "ok", "heartbeats": heartbeats}),
        Err(err) => json!({"status": "error", "error": err.to_string()})
    }
}

#[post("/heartbeat", format = "json", data = "<hb>")]
fn heartbeat(hb: Json<Heartbeat>, con: DbConn) -> JsonValue {
    match Heartbeat::insert(hb.0, con) {
        Ok(s) => json!({"status": "ok", "size": s}),
        Err(err) => json!({"status": "error", "error": err.to_string()})
    }
}

fn main() {
    rocket::ignite()
        .attach(DbConn::fairing())
        .mount("/", routes![
            get_challenge,
            heartbeats,
            heartbeat
        ])
        .launch();
}
