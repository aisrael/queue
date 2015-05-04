#![feature(path_ext, libc)]

extern crate getopts;
extern crate unix_socket;

extern crate libc;
use libc::size_t;
use libc::c_char;
use libc::c_int;
use libc::mode_t;

use std::env;
use std::ffi::CString;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::BufRead;
use std::io::BufReader;
use std::io::LineWriter;
use std::io::Write;
use std::net::Shutdown;
use std::path::Path;

// unstable
use std::fs::PathExt;

// external
use getopts::Options;

use unix_socket::{UnixListener, UnixStream};

const PATH: &'static str = "/tmp/queue";

// TODO Move to its own file
mod sys {

extern crate libc;

    use libc::size_t;
    use libc::c_char;
    use libc::c_int;
    use libc::mode_t;

    extern "system" {
        pub fn mkfifo(path: *const c_char, mode: mode_t) -> c_int;
        pub fn unlink(path: *const c_char) -> c_int;
    }

}

fn mkfifo(path: &str, mode: i16) -> i32 {
    let c_path = CString::new(path).unwrap();
    let p_path = c_path.as_ptr();
    unsafe {
        return sys::mkfifo(p_path, 0o666);
    }
}

fn unlink(path: &str) -> i32 {
    println!("unlink(\"{}\")", path);
    let c_path = CString::new(path).unwrap();
    let p_path = c_path.as_ptr();
    unsafe {
        return sys::unlink(p_path);
    }
}

fn print_usage(program: &str, opts: Options) {
  let brief = format!("Usage: {} [options]", program);
  println!("{}", opts.usage(&brief));
}

fn read_lines(file: &File) {
    loop {
        let mut reader = BufReader::new(file);
        match reader.lines().next() {
            Some(r) => {
                let s = r.unwrap_or_else(|e| panic!(e.to_string()));
                println!("Got \"{}\"", s);
            }
            None => {}
        }
    }
    println!("loop {{}} ended");
}

fn server(path: &str) {
    let mut f = File::open(path).unwrap_or_else(
        |e| panic!("File::open(\"path\") error: {}", e.to_string())
    );
    println!("\"{}\" opened", path);
    read_lines(&f);
}

fn queue(s: &str) {
    let file = OpenOptions::new()
            .write(true)
            .open(PATH).unwrap_or_else(
                |e| panic!("OpenOptions::open() error: {}", e.to_string())
            );
    let mut writer = LineWriter::new(file);
    write!(writer, "{}", s);
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut opts = Options::new();
    opts.optflag("h", "help", "print this help menu");
    opts.optflag("c", "command", "execute the given command on the enqueued arguments");

    let matches = opts.parse(&args[1..]).unwrap_or_else(
        |e| panic!(e.to_string())
    );

    if matches.opt_present("h") {
        print_usage(&args[0], opts);
        return;
    }

    if Path::new(PATH).exists() {
        println!("\"{}\" exists", PATH);
        if !matches.free.is_empty() {
            queue(&matches.free[0]);
        } else {
            print_usage(&args[0], opts);
        }
    } else {
        let res = mkfifo(PATH, 0o666);
        println!("mkfifo(PATH, 0o666) => {}", res);
        // let listener = unix_socket::UnixListener::bind(PATH).unwrap_or_else(
        //     |e| panic!("UnixListener::bind error: {}", e.to_string())
        // );
        server(PATH);
        unlink(PATH);
    }

}
