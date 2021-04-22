use diesel::{self, insert_into, prelude::*, result::QueryResult};
use serde::{Deserialize, Serialize};

use crate::schema::heartbeats;
use crate::schema::heartbeats::dsl::heartbeats as all_heartbeats;

use crate::DbConn;

#[derive(Serialize, Deserialize, Queryable, Insertable, Debug, Clone)]
#[table_name = "heartbeats"]
pub struct Heartbeat {
    pub id: Option<i32>,
    pub cpu_usage: f32,
    pub cpu_total: f32,
    pub mem_usage: f32,
    pub mem_total: f32,
    pub client_timestamp: i32,
    pub server_timestamp: i32,
}

impl Heartbeat {
    pub fn all(conn: DbConn) -> QueryResult<Vec<Heartbeat>> {
        all_heartbeats.load::<Heartbeat>(&conn.0)
    }

    pub fn insert(heartbeat: Heartbeat, conn: DbConn) -> QueryResult<usize> {
        insert_into(all_heartbeats)
            .values(&heartbeat)
            .execute(&conn.0)
    }
}
