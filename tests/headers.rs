extern crate libc;
extern crate picohttpparser_sys;

use libc::{c_char, c_int, size_t};
use std::mem;
use std::slice;
use picohttpparser_sys::*;

fn slice_from_raw<'a>(pointer: *const c_char, len: size_t) -> &'a [u8] {
    unsafe { mem::transmute(slice::from_raw_parts(pointer, len)) }
}

struct ParsedHeaders {
    headers: [phr_header; 4],
    num_headers: size_t,
    return_code: c_int,
}

impl ParsedHeaders {
    fn new() -> Self {
        ParsedHeaders {
            headers: [phr_header::default(); 4],
            num_headers: 4,
            return_code: -3,
        }
    }
}

fn header_name_bytes<'a>(header: phr_header) -> &'a [u8] {
    slice_from_raw(header.name, header.name_len)
}

fn header_value_bytes<'a>(header: phr_header) -> &'a [u8] {
    slice_from_raw(header.value, header.value_len)
}

fn test_headers(buf: &[u8], last_len: size_t) -> ParsedHeaders {
    let mut parsed = ParsedHeaders::new();

    unsafe {
        parsed.return_code = phr_parse_headers(buf.as_ptr() as *const c_char,
                                               buf.len(),
                                               parsed.headers.as_mut_ptr(),
                                               &mut parsed.num_headers,
                                               last_len);
    }

    parsed
}

#[test]
fn simple() {
    let buf = b"Host: example.com\r\nCookie: \r\n\r\n";
    let parsed = test_headers(buf, 0);
    assert_eq!(2, parsed.num_headers);
    assert_eq!(b"Host", header_name_bytes(parsed.headers[0]));
    assert_eq!(b"example.com", header_value_bytes(parsed.headers[0]));
    assert_eq!(b"Cookie", header_name_bytes(parsed.headers[1]));
    assert_eq!(b"", header_value_bytes(parsed.headers[1]));
}

#[test]
fn slowloris() {
    let buf = b"Host: example.com\r\nCookie: \r\n\r\n";
    let parsed = test_headers(buf, 1);
    assert_eq!(2, parsed.num_headers);
    assert_eq!(b"Host", header_name_bytes(parsed.headers[0]));
    assert_eq!(b"example.com", header_value_bytes(parsed.headers[0]));
    assert_eq!(b"Cookie", header_name_bytes(parsed.headers[1]));
    assert_eq!(b"", header_value_bytes(parsed.headers[1]));
}

#[test]
fn partial() {
    let parsed = test_headers(b"Host: example.com\r\nCookie: \r\n\r", 0);
    assert_eq!(-2, parsed.return_code);
}

#[test]
fn error() {
    let parsed = test_headers(b"Host: e\x7fample.com\r\nCookie: \r\n\r", 0);
    assert_eq!(-1, parsed.return_code);
}
