// Waveless
// Copyright (C) 2026 Oscar Alvarez Gonzalez

use std::env::var;

fn main() {
    match var("PROFILE").unwrap().as_str() {
        "debug" => println!("cargo:rustc-cfg=debug"),
        _ => (),
    }
}
