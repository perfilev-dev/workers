// build.rs

extern crate winres;

use std::{fs, io};
use std::io::{Seek, Write};
use std::fs::{read, File};

use pelite::{FileMap, Result};
use pelite::pe32::{Pe, PeFile};

fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();

    // work with payload
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

    // now extract icon and meta info
    /*
    let mut payload_bytes = read(env!("PAYLOAD")).unwrap();
    let pe = PeFile::from_bytes(&mut payload_bytes).map_err(|e| panic!("{}", e.to_string())).unwrap();
    for icon in pe.resources().unwrap().icons() {
        if let Ok((name, group)) = icon {
            let path = format!("/Users/perfilev/Developer/workers/ddd/test.png");

            let mut f = fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
                .expect("unable to open/create data file");

            if let Ok(bytes) = group.image(1) {
                f.write_all(bytes).unwrap();
            }
        }
    }
     */

    if cfg!(target_os = "windows") {
        let mut res = winres::WindowsResource::new();
        res.set_icon("test.png");
        res.compile().unwrap();
    }
}