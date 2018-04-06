#![cfg_attr(feature = "nightly", feature(cfg_target_feature))]
extern crate gcc;

#[cfg(feature = "nightly")]
fn is_enable_sse() -> bool {
    cfg!(target_feature = "sse4.2")
}

#[cfg(not(feature = "nightly"))]
fn is_enable_sse() -> bool {
    false
}

fn main() {
    let mut config = gcc::Config::new();
    config.file("deps/picohttpparser/picohttpparser.c");
    config.include("deps/picohttpparser");

    if is_enable_sse() {
        config.flag("-msse4");
    }

    config.compile("libpicohttpparser.a");

    println!(
        "cargo:rustc-link-search=native={}",
        env!("CARGO_MANIFEST_DIR")
    );
    println!("cargo:rustc-link-lib=static=picohttpparser");
}
