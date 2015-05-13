#![feature(path_ext, libc)]

extern crate getopts;
extern crate unix_socket;
extern crate libc;

mod sys;

use std::env;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, LineWriter, stdout, Write};
use std::path::Path;
use std::process::Command;

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

#[allow(unused_must_use)]
fn handle(s: String) {
    match Command::new("sh").arg("-c").arg(s).output() {
        Ok(output) =>
        {
            std::io::stdout().write(&*output.stdout);
        },
        Err(e) => {
            panic!("failed to execute process: {}", e);
        }
    }
}

fn server(path: &str) {
    loop {
        match File::open(path) {
            Ok(file) => {
                let reader = BufReader::new(&file);
                for line in reader.lines() {
                    handle(line.unwrap_or_else(|e| panic!(e.to_string())));
                }
            },
            Err(e) => { panic!("File::open(\"path\") error: {}", e.to_string()) }
        }
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
        if !matches.free.is_empty() {
            queue(&matches.free.connect(" "));
        } else {
            print_usage(&args[0], opts);
        }
    } else {
        let res = sys::mkfifo(PATH, 0o666);
        unsafe { signal(SIGINT, goodbye as u64); }
        server(PATH);
    }

}

extern fn goodbye() {
    sys::unlink(PATH);
    unsafe { exit(1); }
}
