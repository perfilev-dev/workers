#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate diesel;
#[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_contrib;
#[macro_use] extern crate error_chain;
#[macro_use] extern crate lazy_static;

mod error;
mod heartbeat;
mod schema;
mod expiring;
mod secret;
mod tokens;

use rocket_contrib::json::{Json, JsonValue};
use heartbeat::Heartbeat;
use tokens::Token;
use std::net::SocketAddr;
use serde::{Deserialize, Serialize};
use shared::{challenge::Challenge, api::*};
use expiring::ExpiringData;
use std::time::Duration;
use rocket::response::status::BadRequest;
use std::thread;
use rocket::fairing::AdHoc;
use std::thread::sleep;

#[database("sqlite_database")]
pub struct DbConn(diesel::SqliteConnection);


#[get("/challenge")]
fn get_challenge() -> Json<ChallengeResponse> {
    let challenge = Challenge::new();
    let serialized = serde_json::to_string(&challenge).unwrap();

    let ttl = Duration::from_secs({
        if cfg!(debug_assertions) {
            60      // 1 min for debug
        } else {
            1200    // 20 min for prod
        }
    });

    // generate expiring token
    let expiring = ExpiringData::new(&serialized, ttl);
    let token = secret::encrypt(expiring).unwrap();

    Json(ChallengeResponse { challenge, token })
}


#[post("/register", format = "json", data = "<req>")]
fn register(req: Json<RegisterParameters>, remote_addr: SocketAddr, con: DbConn) -> Result<Json<RegisterResponse>, BadRequest<String>> {
    let expiring = secret::decrypt::<ExpiringData>(&req.token)
        .map_err(|e| BadRequest(Some(e.to_string())))?;

    // check expired.
    if expiring.is_expired() {
        return Err(BadRequest(Some("expired".to_string())));
    }

    // check token in db with expiration time!
    let count = Token::find(&expiring.data, &con)
        .map_err(|e| BadRequest(Some(e.to_string())))?;

    if count > 0 {
        return Err(BadRequest(Some("used".to_string())));
    }

    // verify solution!
    let challenge : Challenge = serde_json::from_str(&expiring.data)
        .map_err(|e| BadRequest(Some(e.to_string())))?;

    // put token to db with expiration time!
    Token::insert(Token::new(&challenge.bytes, expiring.expires_on), &con)
        .map_err(|e| BadRequest(Some(e.to_string())))?;

    if !challenge.check(req.solution) {
        return Err(BadRequest(Some("bad solution".to_string())));
    }

    let ttl = Duration::from_secs({
        if cfg!(debug_assertions) {
            600      // 10 min for debug
        } else {
            86400    // 24 hours for prod
        }
    });

    // all's ok, generate token and encrypt it
    let expiring2 = ExpiringData::new(&Challenge::new().bytes, ttl);
    let token = secret::encrypt(expiring)
        .map_err(|e| BadRequest(Some(e.to_string())))?;

    Ok(Json(RegisterResponse { token }))
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
        .attach(AdHoc::on_launch("Token Cleaner", |r| {
            let con = DbConn::get_one(r).unwrap();

            thread::spawn(move || {
                loop {
                    sleep(Duration::from_secs(5));
                    if let Err(err) = Token::remove_old(&con) {
                        println!("Unable to remove old tokens: {}", err.to_string());
                    }
                }
            });
        }))
        .mount("/", routes![
            get_challenge,
            register,
            heartbeats,
            heartbeat
        ])
        .launch();
}
