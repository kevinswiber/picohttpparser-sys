extern crate gcc;

fn main() {
    gcc::Config::new()
        .file("deps/picohttpparser/picohttpparser.c")
        .include("deps/picohttpparser")
        .compile("libpicohttpparser.a");

    println!("cargo:rustc-link-search=native={}",
             env!("CARGO_MANIFEST_DIR"));
    println!("cargo:rustc-link-lib=static=picohttpparser");
}
