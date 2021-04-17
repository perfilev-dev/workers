use diesel::{self, result::QueryResult, prelude::*, insert_into};

mod schema {
    table! {
        heartbeats {
            id -> Nullable<Integer>,
            foo -> Integer,
        }
    }
}

use self::schema::heartbeats;
use self::schema::heartbeats::dsl::{heartbeats as all_heartbeats};
use serde::{Serialize, Deserialize};

use crate::DbConn;

#[derive(Serialize, Deserialize, Queryable, Insertable, Debug, Clone)]
#[table_name="heartbeats"]
pub struct Heartbeat {
    pub id: Option<i32>,
    pub foo: i32
}

impl Heartbeat {

    pub fn insert(heartbeat: Heartbeat, conn: DbConn) -> QueryResult<usize> {
        insert_into(all_heartbeats).values(&heartbeat).execute(&conn.0)
    }

}