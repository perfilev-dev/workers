use diesel::{self, insert_into, prelude::*, result::QueryResult};
use serde::{Deserialize, Serialize};

use crate::schema::workers;
use crate::schema::workers::dsl::workers as all_workers;

use crate::DbConn;

#[derive(Serialize, Deserialize, Queryable, Insertable, Debug, Clone)]
#[table_name = "workers"]
pub struct Worker {
    pub id: Option<i32>,
    pub token: String,
    pub cpu_total: f32,
    pub mem_total: f32,
    pub client_timestamp: i32,
    pub server_timestamp: i32,
}

impl Worker {
    pub fn all(conn: DbConn) -> QueryResult<Vec<Worker>> {
        all_workers.load::<Worker>(&conn.0)
    }

    pub fn insert(worker: Worker, conn: DbConn) -> QueryResult<usize> {
        insert_into(all_workers)
            .values(&worker)
            .execute(&conn.0)
    }
}
