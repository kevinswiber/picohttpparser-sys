#![feature(cfg_target_feature)]
extern crate gcc;

fn main() {
    let mut config = gcc::Config::new();
    config.file("deps/picohttpparser/picohttpparser.c");
    config.include("deps/picohttpparser");

    if cfg!(target_feature = "sse4.1") || cfg!(target_feature = "sse4.2") {
        config.flag("-msse4");
    }

    config.compile("libpicohttpparser.a");

    println!("cargo:rustc-link-search=native={}",
             env!("CARGO_MANIFEST_DIR"));
    println!("cargo:rustc-link-lib=static=picohttpparser");
}
