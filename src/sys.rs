extern crate libc;

use libc::c_char;
use libc::c_int;
use libc::mode_t;

extern "system" {
    pub fn mkfifo(path: *const c_char, mode: mode_t) -> c_int;
    pub fn unlink(path: *const c_char) -> c_int;
}
