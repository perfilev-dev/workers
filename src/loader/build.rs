// build.rs

extern crate winres;

use std::fs;

fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();

    let path = format!("payload");
    let out_path = format!("{}/{}", out_dir, path);

    let mut out_file = fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(&out_path)
        .expect("unable to open/create data file");

    // 1.


    if cfg!(target_os = "windows") {
        let mut res = winres::WindowsResource::new();
        res.set_icon("test.ico");
        res.compile().unwrap();
    }
}