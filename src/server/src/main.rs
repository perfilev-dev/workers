#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate lazy_static;

use std::fs;
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use rocket::response::status::BadRequest;
use rocket_contrib::json::{Json, JsonValue};

use binaries::Binary;
use expiring::ExpiringData;
use heartbeat::Heartbeat;
use worker::Worker;
use shared::utils::sha256;
use shared::{api::*, challenge::Challenge};
use std::fs::{read, File};
use std::io::Write;
use tokens::Token;
use tokens::TokenFairing;

mod binaries;
mod expiring;
mod heartbeat;
mod schema;
mod secret;
mod tokens;
mod worker;

#[database("sqlite_database")]
pub struct DbConn(diesel::SqliteConnection);

///
/// Registering: get challenge, solve, send solution!
///

#[get("/challenge")]
fn challenge() -> Json<ChallengeResponse> {
    let challenge = Challenge::new();
    let serialized = serde_json::to_string(&challenge).unwrap();

    let ttl = Duration::from_secs({
        if cfg!(debug_assertions) {
            60 // 1 min for debug
        } else {
            1200 // 20 min for prod
        }
    });

    // generate expiring token
    let expiring = ExpiringData::new(&serialized, ttl);
    let token = secret::encrypt(expiring).unwrap();

    Json(ChallengeResponse { challenge, token })
}

#[post("/register", format = "json", data = "<req>")]
fn register(
    req: Json<RegisterParameters>,
    con: DbConn,
) -> Result<Json<RegisterResponse>, BadRequest<String>> {
    let expiring =
        secret::decrypt::<ExpiringData>(&req.token).map_err(|e| BadRequest(Some(e.to_string())))?;

    // check expired.
    if expiring.is_expired() {
        return Err(BadRequest(Some("expired".to_string())));
    }

    // extract challenge
    let challenge: Challenge =
        serde_json::from_str(&expiring.data).map_err(|e| BadRequest(Some(e.to_string())))?;

    // check token in db with expiration time!
    let count = Token::find(&challenge.bytes, &con).map_err(|e| BadRequest(Some(e.to_string())))?;

    if count > 0 {
        return Err(BadRequest(Some("used".to_string())));
    }

    // put token to db with expiration time!
    Token::insert(Token::new(&challenge.bytes, expiring.expires_on), &con)
        .map_err(|e| BadRequest(Some(e.to_string())))?;

    // and check solution!
    if !challenge.check(req.solution) {
        return Err(BadRequest(Some("bad solution".to_string())));
    }

    let ttl = Duration::from_secs({
        if cfg!(debug_assertions) {
            600 // 10 min for debug
        } else {
            86400 // 24 hours for prod
        }
    });

    // all's ok, generate token and encrypt it
    let expiring2 = ExpiringData::new(&Challenge::new().bytes, ttl);
    let token = secret::encrypt(expiring2).map_err(|e| BadRequest(Some(e.to_string())))?;

    // save worker in db!
    Worker::insert(Worker {
        id: None,
        token: token.to_string(),
        cpu_total: req.cpu_total,
        mem_total: req.mem_total,
        client_timestamp: req.timestamp,
        server_timestamp: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i32
    }, con).map_err(|e| BadRequest(Some(e.to_string())))?;;

    Ok(Json(RegisterResponse { token }))
}

///
/// Workers method, that requires token!
///

#[get("/w/client/info")]
fn client_info(con: DbConn) -> Result<Json<ClientInfo>, BadRequest<String>> {
    let binary = binaries::Binary::last(con).map_err(|e| BadRequest(Some(e.to_string())))?;

    Ok(Json(ClientInfo {
        sha256: binary.sha256,
    }))
}

#[get("/w/client/download")]
fn client_download(con: DbConn) -> Result<Json<UploadParameters>, BadRequest<String>> {
    let binary = binaries::Binary::last(con).map_err(|e| BadRequest(Some(e.to_string())))?;

    let bytes = read(binary.sha256).map_err(|e| BadRequest(Some(e.to_string())))?;

    Ok(Json(UploadParameters {
        base64: base64::encode(&bytes),
        sign: binary.signature,
    }))
}

#[post("/w/heartbeat", format = "json", data = "<hb>")]
fn heartbeat(hb: Json<HeartbeatParameters>, con: DbConn) -> Result<Json<HeartbeatResponse>, BadRequest<String>> {
    Heartbeat::insert(Heartbeat {
        id: None,
        token: hb.token.to_string(),
        cpu_usage: hb.cpu_usage,
        mem_usage: hb.mem_usage,
        client_timestamp: hb.timestamp,
        server_timestamp: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i32
    }, con).map_err(|e| BadRequest(Some(e.to_string())))?;

    Ok(Json(HeartbeatResponse { success: true }))
}

///
/// Controllers method, auth by PKI! TODO
///

#[post("/c/binary", format = "json", data = "<upload>")]
fn upload_binary(
    upload: Json<UploadParameters>,
    con: DbConn,
) -> Result<Json<UploadResponse>, BadRequest<String>> {
    let bytes = base64::decode(&upload.base64).map_err(|e| BadRequest(Some(e.to_string())))?;

    // verify binary against signature?
    let sign_ok =
        shared::utils::verify_sign(&bytes, &upload.sign, &shared::utils::KEY.lock().unwrap())
            .map_err(|e| BadRequest(Some(e.to_string())))?;

    if !sign_ok {
        return Err(BadRequest(Some("wrong sign".to_string())));
    }

    let digest = hex::encode(sha256(&bytes));
    let binary = Binary {
        id: None,
        sha256: digest.to_string(),
        signature: upload.sign.to_string(),
    };

    // store in db...
    Binary::insert(binary, &con).map_err(|e| BadRequest(Some(e.to_string())))?;

    // also on fs by hash.
    let mut file = File::create(&digest).unwrap();
    file.write_all(&bytes).unwrap();

    Ok(Json(UploadResponse { sha256: digest }))
}

#[get("/c/heartbeats")]
fn heartbeats(con: DbConn) -> JsonValue {
    match Heartbeat::all(con) {
        Ok(heartbeats) => json!({"status": "ok", "heartbeats": heartbeats}),
        Err(err) => json!({"status": "error", "error": err.to_string()}),
    }
}

///
/// Service methods, not exposed! Only for redirects
///

#[get("/error?<message>")]
fn error(message: String) -> BadRequest<String> {
    println!("error: {}", message);
    BadRequest(Some(message))
}

fn main() {
    rocket::ignite()
        .attach(DbConn::fairing())
        .attach(TokenFairing::new())
        .mount(
            "/",
            routes![
                challenge,
                register,
                heartbeats,
                heartbeat,
                error,
                upload_binary,
                client_info,
                client_download
            ],
        )
        .launch();
}
