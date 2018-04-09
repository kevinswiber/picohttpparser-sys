extern crate cc;

#[cfg(feature = "sse4")]
fn is_enable_sse() -> bool {
    true
}

#[cfg(not(feature = "sse4"))]
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
