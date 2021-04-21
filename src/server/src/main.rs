#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate diesel;
#[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_contrib;
#[macro_use] extern crate lazy_static;

use std::fs;
use std::path::Path;
use std::time::Duration;

use rocket::response::status::BadRequest;
use rocket_contrib::json::{Json, JsonValue};

use binaries::Binary;
use expiring::ExpiringData;
use heartbeat::Heartbeat;
use shared::{api::*, challenge::Challenge};
use shared::utils::sha256;
use tokens::Token;
use tokens::TokenFairing;

mod binaries;
mod heartbeat;
mod schema;
mod expiring;
mod secret;
mod tokens;

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
fn register(req: Json<RegisterParameters>, con: DbConn) -> Result<Json<RegisterResponse>, BadRequest<String>> {
    let expiring = secret::decrypt::<ExpiringData>(&req.token)
        .map_err(|e| BadRequest(Some(e.to_string())))?;

    // check expired.
    if expiring.is_expired() {
        return Err(BadRequest(Some("expired".to_string())));
    }

    // extract challenge
    let challenge : Challenge = serde_json::from_str(&expiring.data)
        .map_err(|e| BadRequest(Some(e.to_string())))?;

    // check token in db with expiration time!
    let count = Token::find(&challenge.bytes, &con)
        .map_err(|e| BadRequest(Some(e.to_string())))?;

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
            600      // 10 min for debug
        } else {
            86400    // 24 hours for prod
        }
    });

    // all's ok, generate token and encrypt it
    let expiring2 = ExpiringData::new(&Challenge::new().bytes, ttl);
    let token = secret::encrypt(expiring2)
        .map_err(|e| BadRequest(Some(e.to_string())))?;

    Ok(Json(RegisterResponse { token }))
}


///
/// Workers method, that requires token!
///

#[post("/w/client/info")]
fn client_info(con: DbConn) -> Result<Json<ClientInfo>, BadRequest<String>> {
    let binary = binaries::Binary::last(con)
        .map_err(|e| BadRequest(Some(e.to_string())))?;

    Ok(Json(ClientInfo {
        sha256: binary.sha256
    }))
}

#[post("/w/heartbeat", format = "json", data = "<hb>")]
fn heartbeat(hb: Json<Heartbeat>, con: DbConn) -> JsonValue {
    match Heartbeat::insert(hb.0, con) {
        Ok(s) => json!({"status": "ok", "size": s}),
        Err(err) => json!({"status": "error", "error": err.to_string()})
    }
}


///
/// Controllers method, auth by PKI! TODO
///

#[post("/c/binary", format = "json", data = "<upload>")]
fn upload_binary(upload: Json<UploadParameters>, con: DbConn) -> Result<Json<UploadResponse>, BadRequest<String>> {
    let bytes = base64::decode(&upload.base64)
        .map_err(|e| BadRequest(Some(e.to_string())))?;

    // verify binary against signature?
    let sign_ok = shared::utils::verify_sign(&bytes, &upload.sign, &shared::utils::KEY.lock().unwrap())
        .map_err(|e| BadRequest(Some(e.to_string())))?;

    if !sign_ok {
        return Err(BadRequest(Some("wrong sign".to_string())));
    }

    let binary = Binary {
        id: None,
        sha256: hex::encode(sha256(&bytes)),
        signature: upload.sign.to_string()
    };

    Binary::insert(binary, &con)
        .map_err(|e| BadRequest(Some(e.to_string())))?;

    Ok(Json(UploadResponse { sha256: hex::encode(sha256(&bytes)) }))
}

#[get("/c/heartbeats")]
fn heartbeats(con: DbConn) -> JsonValue {
    match Heartbeat::all(con) {
        Ok(heartbeats) => json!({"status": "ok", "heartbeats": heartbeats}),
        Err(err) => json!({"status": "error", "error": err.to_string()})
    }
}


///
/// Service methods, not exposed! Only for redirects
///

#[get("/error?<message>")]
fn error(message: String) -> BadRequest<String> {
    BadRequest(Some(message))
}


fn main() {
    rocket::ignite()
        .attach(DbConn::fairing())
        .attach(TokenFairing::new())
        .mount("/", routes![
            challenge,
            register,
            heartbeats,
            heartbeat,
            error,
            upload_binary
        ])
        .launch();
}
