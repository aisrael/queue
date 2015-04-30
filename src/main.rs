#![feature(path_ext)]

extern crate getopts;
extern crate unix_socket;

use std::env;
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

fn print_usage(program: &str, opts: Options) {
  let brief = format!("Usage: {} [options]", program);
  println!("{}", opts.usage(&brief));
}

fn read_lines(unix_stream: UnixStream) {
    let mut reader = BufReader::new(unix_stream);
    loop {
        let s = &mut String::new();
        match reader.read_line(s) {
            Err(e) => panic!("reader.read_line error: {}", e.to_string()),
            Ok(n) => {
                if n == 0 {
                    break;
                } else {
                    println!("Got {} bytes: \"{}\"", n, s);
                }
            }
        }
    }
    println!("loop {{}} ended");
}

fn server(listener: UnixListener) {
    loop {
        match listener.incoming().next() {
            Some(result) => {
                match result {
                    Err(e) => panic!("UnixListener::bind error: {}", e.to_string()),
                    Ok(mut unix_stream) => {
                        read_lines(unix_stream);
                    },
                }
            },
            None => { }
        }
        println!("listener.incoming().next() done");
    }
}

fn queue(s: &str) {
    match UnixStream::connect(PATH) {
        Err(e) => { panic!(e.to_string()) },
        Ok(unix_stream) => {
            let mut writer = LineWriter::new(unix_stream);
            write!(writer, "{}", s);
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut opts = Options::new();
    opts.optflag("h", "help", "print this help menu");
    opts.optflag("c", "command", "execute the given command on the enqueued arguments");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(e) => { panic!(e.to_string()) }
    };

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
        match unix_socket::UnixListener::bind(PATH) {
            Err(e) => panic!("UnixListener::bind error: {}", e.to_string()),
            Ok(mut listener)  => {
                server(listener);
            }
        }
    }

}
