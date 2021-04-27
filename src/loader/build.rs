// build.rs

extern crate winres;

use std::fs;

fn main() {

    // 1.


    if cfg!(target_os = "windows") {
        let mut res = winres::WindowsResource::new();
        res.set_icon("test.ico");
        res.compile().unwrap();
    }
}