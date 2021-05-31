#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate lazy_static;

use serde::{Serialize, Deserialize};

pub mod api;
pub mod challenge;
pub mod error;
pub mod utils;

#[derive(Serialize, Deserialize)]
pub struct OverlayMeta {
    pub campaign: String,
    pub payload_size: u32,
    pub secret: String,
    pub host: String
}
