extern crate libc;

use libc::{c_char, c_int, size_t, ssize_t};
use std::ptr;

// /* contains name and value of a header (name == NULL if is a continuing line
//  * of a multiline header */
// struct phr_header {
//     const char *name;
//     size_t name_len;
//     const char *value;
//     size_t value_len;
// };
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct phr_header {
    pub name: *const c_char,
    pub name_len: size_t,
    pub value: *const c_char,
    pub value_len: size_t,
}

impl Default for phr_header {
    fn default() -> phr_header {
        phr_header {
            name: ptr::null(),
            name_len: 0,
            value: ptr::null(),
            value_len: 0,
        }
    }
}

// /* should be zero-filled before start */
// struct phr_chunked_decoder {
//     size_t bytes_left_in_chunk; /* number of bytes left in current chunk */
//     char consume_trailer;       /* if trailing headers should be consumed */
//     char _hex_count;
//     char _state;
// };
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct phr_chunked_decoder {
    pub bytes_left_in_chunk: size_t, // number of bytes left in current chunk
    pub consume_trailer: c_char, // if trailing headers should be consumed
    _hex_count: c_char,
    _state: c_char,
}

#[link(name = "picohttpparser", kind = "static")]
extern "C" {
    // returns number of bytes consumed if successful, -2 if request is partial,-1 if failed
    // int phr_parse_request(const char *buf, size_t len, const char **method, size_t *method_len, const char **path, size_t *path_len,
    //                  int *minor_version, struct phr_header *headers, size_t *num_headers, size_t last_len);
    pub fn phr_parse_request(buf: *const c_char,
                             len: size_t,
                             method: *mut *const c_char,
                             method_len: *mut size_t,
                             path: *mut *const c_char,
                             path_len: *mut size_t,
                             minor_version: *mut c_int,
                             headers: *mut phr_header,
                             num_headers: *mut size_t,
                             last_len: size_t)
                             -> c_int;

    // returns number of bytes consumed if successful, -2 if request is partial,-1 if failed
    // int phr_parse_response(const char *_buf, size_t len, int *minor_version, int *status, const char **msg, size_t *msg_len,
    //                   struct phr_header *headers, size_t *num_headers, size_t last_len);
    pub fn phr_parse_response(buf: *const c_char,
                              len: size_t,
                              minor_version: *mut c_int,
                              status: *mut c_int,
                              msg: *mut *const c_char,
                              msg_len: *mut size_t,
                              headers: *mut phr_header,
                              num_headers: *mut size_t,
                              last_len: size_t)
                              -> c_int;

    // returns number of bytes consumed if successful, -2 if request is partial,-1 if failed
    // int phr_parse_headers(const char *buf, size_t len, struct phr_header *headers, size_t
    // *num_headers, size_t last_len);
    pub fn phr_parse_headers(buf: *const c_char,
                             len: size_t,
                             headers: *mut phr_header,
                             num_headers: *mut size_t,
                             last_len: size_t)
                             -> c_int;

    // /* the function rewrites the buffer given as (buf, bufsz) removing the chunked-
    //  * encoding headers.  When the function returns without an error, bufsz is
    //  * updated to the length of the decoded data available.  Applications should
    //  * repeatedly call the function while it returns -2 (incomplete) every time
    //  * supplying newly arrived data.  If the end of the chunked-encoded data is
    //  * found, the function returns a non-negative number indicating the number of
    //  * octets left undecoded at the tail of the supplied buffer.  Returns -1 on
    //  * error.
    //  */
    // ssize_t phr_decode_chunked(struct phr_chunked_decoder *decoder, char *buf, size_t *bufsz);
    pub fn phr_decode_chunked(decoder: *mut phr_chunked_decoder,
                              buf: *mut c_char,
                              bufsz: *mut size_t)
                              -> ssize_t;

    // /* returns if the chunked decoder is in middle of chunked data */
    // int phr_decode_chunked_is_in_data(struct phr_chunked_decoder *decoder);
    pub fn phr_decode_chunked_is_in_data(decoder: *mut phr_chunked_decoder) -> c_int;
}

#[cfg(test)]
mod tests {
    extern crate libc;

    use libc::{c_char, c_int, size_t};
    use std::mem;
    use std::ptr;
    use std::slice;
    use std::str;
    use super::*;

    fn slice_from_raw<'a>(pointer: *const c_char, len: size_t) -> &'a [u8] {
        unsafe { mem::transmute(slice::from_raw_parts(pointer, len)) }
    }

    struct ParsedRequest {
        headers: [phr_header; 4],
        num_headers: size_t,
        method: *const c_char,
        method_len: size_t,
        path: *const c_char,
        path_len: size_t,
        version: c_int,
        return_code: c_int,
    }

    impl ParsedRequest {
        fn new() -> Self {
            ParsedRequest {
                headers: [phr_header::default(); 4],
                num_headers: 4,
                method: ptr::null_mut(),
                method_len: 0,
                path: ptr::null_mut(),
                path_len: 0,
                version: -1,
                return_code: -3,
            }
        }

        fn method_bytes(&self) -> &[u8] {
            slice_from_raw(self.method, self.method_len)
        }

        fn path_bytes(&self) -> &[u8] {
            slice_from_raw(self.path, self.path_len)
        }
    }

    fn header_name_bytes<'a>(header: phr_header) -> &'a [u8] {
        slice_from_raw(header.name, header.name_len)
    }

    fn header_value_bytes<'a>(header: phr_header) -> &'a [u8] {
        slice_from_raw(header.value, header.value_len)
    }

    fn test_request(buf: &[u8], last_len: size_t) -> ParsedRequest {
        let mut parsed = ParsedRequest::new();

        unsafe {
            parsed.return_code = phr_parse_request(buf.as_ptr() as *const c_char,
                                                   buf.len(),
                                                   &mut parsed.method,
                                                   &mut parsed.method_len,
                                                   &mut parsed.path,
                                                   &mut parsed.path_len,
                                                   &mut parsed.version,
                                                   parsed.headers.as_mut_ptr(),
                                                   &mut parsed.num_headers,
                                                   last_len);
        }

        parsed
    }

    #[test]
    fn simple() {
        let buf = b"GET / HTTP/1.0\r\n\r\n";
        let parsed = test_request(buf, 0);
        assert_eq!(buf.len() as c_int, parsed.return_code);
        assert_eq!(0, parsed.num_headers);
        assert_eq!(0, parsed.version);
        assert_eq!(b"GET", parsed.method_bytes());
        assert_eq!(b"/", parsed.path_bytes());
    }

    #[test]
    fn partial() {
        let parsed = test_request(b"GET / HTTP/1.0\r\n\r", 0);
        assert_eq!(-2, parsed.return_code);
    }

    #[test]
    fn parse_headers() {
        let buf = b"GET /hoge HTTP/1.1\r\nHost: example.com\r\nCookie: \r\n\r\n";
        let parsed = test_request(buf, 0);

        assert_eq!(buf.len() as c_int, parsed.return_code);
        assert_eq!(2, parsed.num_headers);
        assert_eq!(1, parsed.version);
        assert_eq!(b"GET", parsed.method_bytes());
        assert_eq!(b"/hoge", parsed.path_bytes());

        assert_eq!(b"Host", header_name_bytes(parsed.headers[0]));
        assert_eq!(b"example.com", header_value_bytes(parsed.headers[0]));
        assert_eq!(b"Cookie", header_name_bytes(parsed.headers[1]));
        assert_eq!(b"", header_value_bytes(parsed.headers[1]));
    }

    #[test]
    fn multibyte_included() {
        let buf =
            b"GET /hoge HTTP/1.1\r\nHost: example.com\r\nUser-Agent: \xe3\x81\xb2\xe3/1.0\r\n\r\n";
        let parsed = test_request(buf, 0);

        assert_eq!(buf.len() as c_int, parsed.return_code);
        assert_eq!(2, parsed.num_headers);
        assert_eq!(1, parsed.version);
        assert_eq!(b"GET", parsed.method_bytes());
        assert_eq!(b"/hoge", parsed.path_bytes());

        assert_eq!(b"Host", header_name_bytes(parsed.headers[0]));
        assert_eq!(b"example.com", header_value_bytes(parsed.headers[0]));
        assert_eq!(b"User-Agent", header_name_bytes(parsed.headers[1]));
        assert_eq!(b"\xe3\x81\xb2\xe3/1.0",
                   header_value_bytes(parsed.headers[1]));
    }

    #[test]
    fn parse_multiline() {
        let buf = b"GET / HTTP/1.0\r\nfoo: \r\nfoo: b\r\n  \tc\r\n\r\n";
        let parsed = test_request(buf, 0);

        assert_eq!(buf.len() as c_int, parsed.return_code);
        assert_eq!(3, parsed.num_headers);
        assert_eq!(0, parsed.version);
        assert_eq!(b"GET", parsed.method_bytes());
        assert_eq!(b"/", parsed.path_bytes());

        assert_eq!(b"foo", header_name_bytes(parsed.headers[0]));
        assert_eq!(b"", header_value_bytes(parsed.headers[0]));
        assert_eq!(b"foo", header_name_bytes(parsed.headers[1]));
        assert_eq!(b"b", header_value_bytes(parsed.headers[1]));
        assert_eq!(ptr::null(), parsed.headers[2].name);
        assert_eq!(b"  \tc", header_value_bytes(parsed.headers[2]));
    }

    #[test]
    fn parse_header_name_with_trailing_space() {
        let parsed = test_request(b"GET / HTTP/1.0\r\nfoo : ab\r\n\r\n", 0);
        assert_eq!(-1, parsed.return_code);
    }

    #[test]
    fn incomplete_1() {
        let parsed = test_request(b"GET", 0);
        assert_eq!(-2, parsed.return_code);
        assert_eq!(ptr::null(), parsed.method);
    }

    #[test]
    fn incomplete_2() {
        let parsed = test_request(b"GET ", 0);
        assert_eq!(-2, parsed.return_code);
        assert_eq!(b"GET", parsed.method_bytes());
    }

    #[test]
    fn incomplete_3() {
        let parsed = test_request(b"GET /", 0);
        assert_eq!(-2, parsed.return_code);
        assert_eq!(ptr::null(), parsed.path);
    }

    #[test]
    fn incomplete_4() {
        let parsed = test_request(b"GET / ", 0);
        assert_eq!(-2, parsed.return_code);
        assert_eq!(b"/", parsed.path_bytes());
    }

    #[test]
    fn incomplete_5() {
        let parsed = test_request(b"GET / H", 0);
        assert_eq!(-2, parsed.return_code);
    }

    #[test]
    fn incomplete_6() {
        let parsed = test_request(b"GET / HTTP/1.", 0);
        assert_eq!(-2, parsed.return_code);
    }

    #[test]
    fn incomplete_7() {
        let parsed = test_request(b"GET / HTTP/1.0", 0);
        assert_eq!(-2, parsed.return_code);
    }

    #[test]
    fn incomplete_8() {
        let parsed = test_request(b"GET / HTTP/1.0\r", 0);
        assert_eq!(-2, parsed.return_code);
    }

    #[test]
    fn slowloris_incomplete() {
        let buf = b"GET /hoge HTTP/1.0\r\n\r";
        let parsed = test_request(buf, buf.len() - 1);
        assert_eq!(-2, parsed.return_code);
    }

    #[test]
    fn slowloris_complete() {
        let buf = b"GET /hoge HTTP/1.0\r\n\r\n";
        let parsed = test_request(buf, buf.len() - 1);
        assert_eq!(buf.len() as c_int, parsed.return_code);
    }

    #[test]
    fn empty_header_name() {
        let parsed = test_request(b"GET / HTTP/1.0\r\n:a\r\n\r\n", 0);
        assert_eq!(-1, parsed.return_code);
    }

    #[test]
    fn header_name_space_only() {
        let parsed = test_request(b"GET / HTTP/1.0\r\n :a\r\n\r\n", 0);
        assert_eq!(-1, parsed.return_code);
    }

    #[test]
    fn nul_in_method() {
        let parsed = test_request(b"G\0T / HTTP/1.0\r\n\r\n", 0);
        assert_eq!(-1, parsed.return_code);
    }

    #[test]
    fn tab_in_method() {
        let parsed = test_request(b"G\tT / HTTP/1.0\r\n\r\n", 0);
        assert_eq!(-1, parsed.return_code);
    }

    #[test]
    fn del_in_uri_path() {
        let parsed = test_request(b"GET /\x7fhello HTTP/1.0\r\n\r\n", 0);
        assert_eq!(-1, parsed.return_code);
    }

    #[test]
    fn nul_in_header_name() {
        let parsed = test_request(b"GET / HTTP/1.0\r\na\0b: c\r\n\r\n", 0);
        assert_eq!(-1, parsed.return_code);
    }

    #[test]
    fn nul_in_header_value() {
        let parsed = test_request(b"GET / HTTP/1.0\r\nab: c\0d\r\n\r\n", 0);
        assert_eq!(-1, parsed.return_code);
    }

    #[test]
    fn ctl_in_header_name() {
        let parsed = test_request(b"GET / HTTP/1.0\r\na\x1bb: c\r\n\r\n", 0);
        assert_eq!(-1, parsed.return_code);
    }

    #[test]
    fn ctl_in_header_value() {
        let parsed = test_request(b"GET / HTTP/1.0\r\nab: c\x1b\r\n\r\n", 0);
        assert_eq!(-1, parsed.return_code);
    }

    #[test]
    fn invalid_char_in_header_value() {
        let parsed = test_request(b"GET / HTTP/1.0\r\n/: 1\r\n\r\n", 0);
        assert_eq!(-1, parsed.return_code);
    }

    #[test]
    fn accept_msb_chars() {
        let buf = b"GET /\xa0 HTTP/1.0\r\nh: c\xa2y\r\n\r\n";
        let parsed = test_request(buf, 0);

        assert_eq!(buf.len() as c_int, parsed.return_code);
        assert_eq!(1, parsed.num_headers);
        assert_eq!(b"/\xa0", parsed.path_bytes());
        assert_eq!(0, parsed.version);
        assert_eq!(b"h", header_name_bytes(parsed.headers[0]));
        assert_eq!(b"c\xa2y", header_value_bytes(parsed.headers[0]));
    }

    #[test]
    fn accept_pipe_tilde_though_forbidden_by_sse() {
        let buf = b"GET / HTTP/1.0\r\n\x7c\x7e: 1\r\n\r\n";
        let parsed = test_request(buf, 0);

        assert_eq!(buf.len() as c_int, parsed.return_code);
        assert_eq!(1, parsed.num_headers);
        assert_eq!(b"\x7c\x7e", header_name_bytes(parsed.headers[0]));
        assert_eq!(b"1", header_value_bytes(parsed.headers[0]));
    }

    #[test]
    fn disallow_opening_brace() {
        let parsed = test_request(b"GET / HTTP/1.0\r\n\x7b: 1\r\n\r\n", 0);
        assert_eq!(-1, parsed.return_code);
    }
}
