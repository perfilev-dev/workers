// build.rs

extern crate winres;

use std::{fs, io};

fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();

    let path = format!("payload");
    let out_path = format!("{}/{}", out_dir, path);

    let mut out_file = fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(&out_path)
        .expect("unable to open/create data file");

    let mut source_file = fs::File::open(env!("PAYLOAD"))
        .expect("unable to find payload");

    io::copy(&mut source_file, &mut out_file)
        .expect("failed to copy data after opening");

    if cfg!(target_os = "windows") {
        let mut res = winres::WindowsResource::new();
        res.set_icon("test.ico");
        res.compile().unwrap();
    }
}