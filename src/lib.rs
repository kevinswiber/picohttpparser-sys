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
