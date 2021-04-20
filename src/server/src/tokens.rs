use diesel::{self, result::QueryResult, prelude::*, insert_into};
use serde::{Serialize, Deserialize};

use crate::schema::tokens;
use crate::schema::tokens::dsl::{tokens as all_tokens};

use crate::DbConn;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize, Deserialize, Queryable, Insertable, Debug, Clone)]
#[table_name="tokens"]
pub struct Token {
    pub id: Option<i32>,
    pub token: String,
    pub expires_on: i32
}

impl Token {

    pub fn new(token: &str, expires_on: SystemTime) -> Token {
        Token {
            id: None,
            token: token.chars().into_iter().take(32).collect(),
            expires_on: expires_on.duration_since(UNIX_EPOCH).unwrap().as_secs() as i32
        }
    }

    pub fn find(token: &str, conn: &DbConn) -> QueryResult<usize> {
        all_tokens.filter(tokens::token.eq(token)).execute(&conn.0)
    }

    pub fn insert(token: Token, conn: &DbConn) -> QueryResult<usize> {
        insert_into(all_tokens).values(&token).execute(&conn.0)
    }

    pub fn remove_old(conn: &DbConn) -> QueryResult<usize> {
        let current = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i32;
        diesel::delete(all_tokens.filter(tokens::expires_on.le(current))).execute(&conn.0)
    }

}