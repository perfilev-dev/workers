use diesel::{self, result::QueryResult, prelude::*, insert_into};
use serde::{Serialize, Deserialize};
use rand::{thread_rng, Rng, RngCore};
use rand::distributions::Alphanumeric;
use sha2::{Sha256, Digest};

use crate::schema::challenges;
use crate::schema::challenges::dsl::{challenges as all_challenges};

use crate::DbConn;
use rocket::http::route::Source::Query;

#[derive(Serialize, Deserialize, Queryable, Insertable, Debug, Clone)]
#[table_name="challenges"]
pub struct Challenge {
    pub id: Option<i32>,
    pub ip: String,
    pub bytes: String,
    pub nonce: i32
}

impl Challenge {

    pub fn new(ip: &str) -> Challenge {
        Challenge {
            id: None,
            ip: ip.to_string(),
            nonce: thread_rng().next_u32() as i32,
            bytes: thread_rng()
                .sample_iter(&Alphanumeric)
                .take(32)
                .map(char::from)
                .collect()
        }
    }

    pub fn all(conn: &DbConn) -> QueryResult<Vec<Challenge>> {
        all_challenges.load::<Challenge>(&conn.0)
    }

    pub fn count_by_ip(ip: &str, conn: &DbConn) -> QueryResult<usize> {
        all_challenges.filter(challenges::ip.eq(ip)).count().execute(&conn.0)
    }

    pub fn remove_first_with_ip(ip: &str, conn: &DbConn) -> QueryResult<usize> {
        let first : Result<Challenge, diesel::result::Error> = all_challenges
            .filter(challenges::ip.eq(ip)).first(&conn.0);

        match first {
            Ok(c) => diesel::delete(all_challenges
                .filter(challenges::id.eq(c.id)))
                .execute(&conn.0),
            Err(e) => QueryResult::Err(e)
        }
    }

    pub fn insert(challenge: Challenge, conn: &DbConn) -> QueryResult<usize> {
        insert_into(all_challenges).values(&challenge).execute(&conn.0)
    }

    pub fn pop_by_bytes(bytes: &str, conn: &DbConn) -> QueryResult<Challenge> {
        let challenge : Result<Challenge, diesel::result::Error> = all_challenges
            .filter(challenges::bytes.eq(bytes)).first(&conn.0);

        // remove also by bytes
        diesel::delete(all_challenges
            .filter(challenges::bytes.eq(bytes)))
            .execute(&conn.0);

        challenge
    }

    pub fn check(&self, solution: i32) -> bool {
        let mut hasher = Sha256::new();
        hasher.update(&self.bytes);
        hasher.update(i32::to_be_bytes(solution));
        let result = hasher.finalize();
        result.ends_with(&i32::to_be_bytes(self.nonce))
    }

}
