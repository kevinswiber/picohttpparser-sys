extern crate libc;
extern crate picohttpparser_sys;

use libc::{c_char, c_int, size_t};
use std::mem;
use std::ptr;
use std::slice;
use std::str;
use picohttpparser_sys::*;

fn slice_from_raw<'a>(pointer: *const c_char, len: size_t) -> &'a [u8] {
    unsafe { mem::transmute(slice::from_raw_parts(pointer, len)) }
}

struct ParsedResponse {
    headers: [phr_header; 4],
    num_headers: size_t,
    msg: *const c_char,
    msg_len: size_t,
    status: c_int,
    version: c_int,
    return_code: c_int,
}

impl ParsedResponse {
    fn new() -> Self {
        ParsedResponse {
            headers: [phr_header::default(); 4],
            num_headers: 4,
            msg: ptr::null_mut(),
            msg_len: 0,
            status: 0,
            version: -1,
            return_code: -3,
        }
    }

    fn msg_bytes(&self) -> &[u8] {
        slice_from_raw(self.msg, self.msg_len)
    }
}

fn header_name_bytes<'a>(header: phr_header) -> &'a [u8] {
    slice_from_raw(header.name, header.name_len)
}

fn header_value_bytes<'a>(header: phr_header) -> &'a [u8] {
    slice_from_raw(header.value, header.value_len)
}

fn test_response(buf: &[u8], last_len: size_t) -> ParsedResponse {
    let mut parsed = ParsedResponse::new();

    unsafe {
        parsed.return_code = phr_parse_response(buf.as_ptr() as *const c_char,
                                                buf.len(),
                                                &mut parsed.version,
                                                &mut parsed.status,
                                                &mut parsed.msg,
                                                &mut parsed.msg_len,
                                                parsed.headers.as_mut_ptr(),
                                                &mut parsed.num_headers,
                                                last_len);
    }

    parsed
}

#[test]
fn simple() {
    let buf = b"HTTP/1.0 200 OK\r\n\r\n";
    let parsed = test_response(buf, 0);
    assert_eq!(buf.len() as c_int, parsed.return_code);
    assert_eq!(0, parsed.num_headers);
    assert_eq!(0, parsed.version);
    assert_eq!(200, parsed.status);
    assert_eq!(b"OK", parsed.msg_bytes());
}

#[test]
fn partial() {
    let parsed = test_response(b"HTTP/1.0 200 OK\r\n\r", 0);
    assert_eq!(-2, parsed.return_code);
}

#[test]
fn parse_headers() {
    let buf = b"HTTP/1.1 200 OK\r\nHost: example.com\r\nCookie: \r\n\r\n";
    let parsed = test_response(buf, 0);
    assert_eq!(buf.len() as c_int, parsed.return_code);
    assert_eq!(2, parsed.num_headers);
    assert_eq!(1, parsed.version);
    assert_eq!(200, parsed.status);
    assert_eq!(b"OK", parsed.msg_bytes());
    assert_eq!(b"Host", header_name_bytes(parsed.headers[0]));
    assert_eq!(b"example.com", header_value_bytes(parsed.headers[0]));
    assert_eq!(b"Cookie", header_name_bytes(parsed.headers[1]));
    assert_eq!(b"", header_value_bytes(parsed.headers[1]));
}

#[test]
fn parse_multiline() {
    let buf = b"HTTP/1.0 200 OK\r\nfoo: \r\nfoo: b\r\n  \tc\r\n\r\n";
    let parsed = test_response(buf, 0);
    assert_eq!(buf.len() as c_int, parsed.return_code);
    assert_eq!(3, parsed.num_headers);
    assert_eq!(0, parsed.version);
    assert_eq!(200, parsed.status);
    assert_eq!(b"OK", parsed.msg_bytes());
    assert_eq!(b"foo", header_name_bytes(parsed.headers[0]));
    assert_eq!(b"", header_value_bytes(parsed.headers[0]));
    assert_eq!(b"foo", header_name_bytes(parsed.headers[1]));
    assert_eq!(b"b", header_value_bytes(parsed.headers[1]));
    assert_eq!(ptr::null(), parsed.headers[2].name);
    assert_eq!(b"  \tc", header_value_bytes(parsed.headers[2]));
}

#[test]
fn internal_server_error() {
    let buf = b"HTTP/1.0 500 Internal Server Error\r\n\r\n";
    let parsed = test_response(buf, 0);
    assert_eq!(buf.len() as c_int, parsed.return_code);
    assert_eq!(0, parsed.num_headers);
    assert_eq!(0, parsed.version);
    assert_eq!(500, parsed.status);
    assert_eq!(b"Internal Server Error", parsed.msg_bytes());
    assert_eq!(b"Internal Server Error".len(), parsed.msg_len);
}

#[test]
fn incomplete_1() {
    let parsed = test_response(b"H", 0);
    assert_eq!(-2, parsed.return_code);
}

#[test]
fn incomplete_2() {
    let parsed = test_response(b"HTTP/1.", 0);
    assert_eq!(-2, parsed.return_code);
}

#[test]
fn incomplete_3() {
    let parsed = test_response(b"HTTP/1.1", 0);
    assert_eq!(-2, parsed.return_code);
    assert_eq!(-1, parsed.version);
}

#[test]
fn incomplete_4() {
    let parsed = test_response(b"HTTP/1.1 ", 0);
    assert_eq!(-2, parsed.return_code);
    assert_eq!(1, parsed.version);
}

#[test]
fn incomplete_5() {
    let parsed = test_response(b"HTTP/1.1 2", 0);
    assert_eq!(-2, parsed.return_code);
    assert_eq!(1, parsed.version);
}

#[test]
fn incomplete_6() {
    let parsed = test_response(b"HTTP/1.1 200", 0);
    assert_eq!(-2, parsed.return_code);
    assert_eq!(0, parsed.status);
}

#[test]
fn incomplete_7() {
    let parsed = test_response(b"HTTP/1.1 200 ", 0);
    assert_eq!(-2, parsed.return_code);
    assert_eq!(200, parsed.status);
}

#[test]
fn incomplete_8() {
    let parsed = test_response(b"HTTP/1.1 200 O", 0);
    assert_eq!(-2, parsed.return_code);
    assert_eq!(ptr::null(), parsed.msg);
}

#[test]
fn incomplete_9() {
    let parsed = test_response(b"HTTP/1.1 200 OK\r", 0);
    assert_eq!(-2, parsed.return_code);
    assert_eq!(ptr::null(), parsed.msg);
}

#[test]
fn incomplete_10() {
    let parsed = test_response(b"HTTP/1.1 200 OK\r\n", 0);
    assert_eq!(-2, parsed.return_code);
    assert_eq!(b"OK", parsed.msg_bytes());
}

#[test]
fn incomplete_11() {
    let parsed = test_response(b"HTTP/1.1 200 OK\n", 0);
    assert_eq!(-2, parsed.return_code);
    assert_eq!(b"OK", parsed.msg_bytes());
}

#[test]
fn incomplete_12() {
    let parsed = test_response(b"HTTP/1.1 200 OK\r\nA: 1\r", 0);
    assert_eq!(-2, parsed.return_code);
    assert_eq!(0, parsed.num_headers);
}

#[test]
fn incomplete_13() {
    let parsed = test_response(b"HTTP/1.1 200 OK\r\nA: 1\r\n", 0);
    assert_eq!(-2, parsed.return_code);
    assert_eq!(1, parsed.num_headers);
    assert_eq!(b"A", header_name_bytes(parsed.headers[0]));
    assert_eq!(b"1", header_value_bytes(parsed.headers[0]));
}

#[test]
fn slowloris_incomplete() {
    let buf = b"HTTP/1.0 200 OK \r\n\r";
    let parsed = test_response(buf, buf.len() - 1);
    assert_eq!(-2, parsed.return_code);
}

#[test]
fn slowloris_complete() {
    let buf = b"HTTP/1.0 200 OK \r\n\r\n";
    let parsed = test_response(buf, buf.len() - 1);
    assert_eq!(buf.len() as c_int, parsed.return_code);
}

#[test]
fn invalid_http_version() {
    let parsed = test_response(b"HTTP/1. 200 OK\r\n\r\n", 0);
    assert_eq!(-1, parsed.return_code);
}

#[test]
fn invalid_http_version_2() {
    let parsed = test_response(b"HTTP/1.2z 200 OK\r\n\r\n", 0);
    assert_eq!(-1, parsed.return_code);
}

#[test]
fn no_status_code() {
    let parsed = test_response(b"HTTP/1.1 OK\r\n\r\n", 0);
    assert_eq!(-1, parsed.return_code);
}
