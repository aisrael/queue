#![feature(path_ext, libc)]

extern crate getopts;
extern crate unix_socket;
extern crate libc;

mod sys;

use std::env;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::BufRead;
use std::io::BufReader;
use std::io::LineWriter;
use std::io::Write;
use std::path::Path;

use libc::exit;
use libc::consts::os::posix88::SIGINT;
use libc::funcs::posix01::signal::signal;

// unstable
use std::fs::PathExt;

// external
use getopts::Options;

// TODO allow this as option
const PATH: &'static str = "/tmp/queue";

fn print_usage(program: &str, opts: Options) {
  let brief = format!("Usage: {} [options]", program);
  println!("{}", opts.usage(&brief));
}

fn handle(result: std::io::Result<String>) {
    let s = result.unwrap_or_else(|e| panic!(e.to_string()));
    println!("Got \"{}\"", s);
}

fn read_lines(file: File) {
    loop {
        let reader = BufReader::new(&file);
        for line in reader.lines() {
            handle(line);
        }
    }
}

fn server(path: &str) {
    match File::open(path) {
        Ok(f) => {
            println!("\"{}\" opened", path);
            read_lines(f);
        },
        Err(e) => { panic!("File::open(\"path\") error: {}", e.to_string()) }
    }
}

fn queue(s: &str) {
    let file = OpenOptions::new()
            .write(true)
            .open(PATH).unwrap_or_else(
                |e| panic!("OpenOptions::open() error: {}", e.to_string())
            );
    let mut writer = LineWriter::new(file);
    write!(writer, "{}\n", s).ok();
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
        let res = sys::mkfifo(PATH, 0o666);
        unsafe { signal(SIGINT, goodbye as u64); }
        println!("mkfifo(PATH, 0o666) => {}", res);
        server(PATH);
    }

}

extern fn goodbye() {
    sys::unlink(PATH);
    unsafe { exit(1); }
}
