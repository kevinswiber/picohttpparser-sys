#![cfg_attr(feature = "nightly", feature(cfg_target_feature))]
extern crate cc;

#[cfg(feature = "nightly")]
fn is_enable_sse() -> bool {
    cfg!(target_feature = "sse4.2")
}

#[cfg(not(feature = "nightly"))]
fn is_enable_sse() -> bool {
    false
}

fn main() {
    let mut build = cc::Build::new();

    if is_enable_sse() {
        build.flag("-msse4");
    }

    build
        .file("deps/picohttpparser/picohttpparser.c")
        .include("deps/picohttpparser")
        .compile("libpicohttpparser.a");
}
