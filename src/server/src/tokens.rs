use diesel::{self, insert_into, prelude::*, result::QueryResult};
use serde::{Deserialize, Serialize};

use crate::schema::tokens;
use crate::schema::tokens::dsl::tokens as all_tokens;

use crate::expiring::ExpiringData;
use crate::{secret, DbConn};
use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::uri::Origin;
use rocket::http::Method;
use rocket::{Data, Request, Rocket};
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::sleep;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Serialize, Deserialize, Queryable, Insertable, Debug, Clone)]
#[table_name = "tokens"]
pub struct Token {
    pub id: Option<i32>,
    pub token: String,
    pub expires_on: i32,
}

impl Token {
    pub fn new(token: &str, expires_on: SystemTime) -> Token {
        Token {
            id: None,
            token: token.chars().into_iter().take(32).collect(),
            expires_on: expires_on.duration_since(UNIX_EPOCH).unwrap().as_secs() as i32,
        }
    }

    pub fn find(token: &str, conn: &DbConn) -> QueryResult<i64> {
        all_tokens
            .filter(tokens::token.eq(token))
            .count()
            .get_result(&conn.0)
    }

    pub fn insert(token: Token, conn: &DbConn) -> QueryResult<usize> {
        insert_into(all_tokens).values(&token).execute(&conn.0)
    }

    pub fn remove_old(conn: &DbConn) -> QueryResult<usize> {
        let current = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i32;
        diesel::delete(all_tokens.filter(tokens::expires_on.le(current))).execute(&conn.0)
    }
}

pub struct TokenFairing {
    con: Arc<Mutex<Option<DbConn>>>,
}

impl TokenFairing {
    pub fn new() -> TokenFairing {
        TokenFairing {
            con: Arc::new(Mutex::new(None)),
        }
    }
}

impl Fairing for TokenFairing {
    fn info(&self) -> Info {
        Info {
            name: "TokenFairing",
            kind: Kind::Launch | Kind::Request,
        }
    }

    fn on_launch(&self, r: &Rocket) {
        *self.con.lock().unwrap() = Some(DbConn::get_one(r).unwrap());

        let con = self.con.clone();
        thread::spawn(move || loop {
            sleep(Duration::from_secs(5));
            let con = con.lock().unwrap();
            if let Some(c) = con.as_ref() {
                if let Err(err) = Token::remove_old(c) {
                    println!("Unable to remove old tokens: {}", err.to_string());
                }
            }
        });
    }

    fn on_request<'r>(&self, r: &mut Request<'r>, _: &Data) {
        if r.uri().path().starts_with("/w/") {
            // check `token` from headers (not expired, not revoked)
            if let Some(token) = r.headers().get("token").last() {
                if let Ok(e) = secret::decrypt::<ExpiringData>(token) {
                    let con = self.con.lock().unwrap();
                    if !e.is_expired() {
                        if let Some(c) = con.as_ref() {
                            if Token::find(&e.data, c).unwrap_or(1) < 1 {
                                return; // not expired and not revoked!
                            }
                        }
                    }
                }
            }

            r.set_uri(Origin::parse("/error?message=register%20needed").unwrap());
            r.set_method(Method::Get)
        } else if r.uri().path().starts_with("/error") {
            r.set_uri(Origin::parse("/not_found").unwrap());
            r.set_method(Method::Get)
        }
    }
}
