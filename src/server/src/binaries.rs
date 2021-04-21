use diesel::{self, result::QueryResult, prelude::*, insert_into};
use serde::{Serialize, Deserialize};

use crate::schema::binaries;
use crate::schema::binaries::dsl::{binaries as all_binaries};

use crate::DbConn;

#[derive(Serialize, Deserialize, Queryable, Insertable, Debug, Clone)]
#[table_name="binaries"]
pub struct Binary {
    pub id: Option<i32>,
    pub sha256: String,
    pub signature: String
}

impl Binary {

    pub fn last(conn: DbConn) -> QueryResult<Binary> {
        all_binaries.order_by(binaries::id).first(&conn.0)
    }

    pub fn insert(binary: Binary, conn: &DbConn) -> QueryResult<usize> {
        insert_into(all_binaries).values(&binary).execute(&conn.0)
    }

}