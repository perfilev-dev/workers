#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate diesel;
#[macro_use] extern crate diesel_migrations;
#[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_contrib;

mod heartbeat;

use serde::{Serialize, Deserialize};
use rocket_contrib::json::{Json, JsonValue};

use heartbeat::Heartbeat;

#[database("sqlite_database")]
pub struct DbConn(diesel::SqliteConnection);


#[post("/heartbeat", format = "json", data = "<hb>")]
fn heartbeat(hb: Json<Heartbeat>, con: DbConn) -> JsonValue {
    println!("{}", hb.0.foo);
    let t = Heartbeat::insert(hb.0, con);
    match t {
        Ok(s) => json!({"status": "ok", "size": s}),
        Err(err) => json!({"status": "error", "error": err.to_string()})
    }


}

fn main() {
    rocket::ignite()
        .attach(DbConn::fairing())
        .mount("/", routes![heartbeat])
        .launch();
}
