#![feature(lookup_host)]

extern crate rustc_serialize;
extern crate libc;
#[macro_use(println_stderr)]
extern crate webextension_protocol as protocol;
#[macro_use]
extern crate js;

mod js_engine;
mod server;
mod helpers;

use std::io::Write;
use std::thread;
use std::sync::mpsc;
use server::{ServerWrapper};
use js_engine::JsEngine;

#[allow(unused_must_use)]
fn main() {

    let (tx, rx) = mpsc::channel();

    let sender_for_io = tx.clone();
    // thread reads from stdin and send messages to main thread
    thread::spawn(move || {
        loop {
            let (io_sender, io_receiver) = mpsc::channel();
            let message = match protocol::read_stdin() {
                Ok(m) => m,
                Err(_) => panic!("nothing more to read"),
            };
            sender_for_io.send((io_sender, message)).unwrap();
            let response = io_receiver.recv().unwrap();
            protocol::write_stdout(response);
        }
    });

    let sender_for_http = tx.clone();

    thread::spawn(move || {
        let mut server = ServerWrapper::new(sender_for_http);
        server.start();
    });

    let mut js_engine = JsEngine::new();

    loop {
        let (io_sender, message) = rx.recv().unwrap();
        let response = js_engine.post_message(message);
        println_stderr!("js returned: {}", response);
        io_sender.send(response).unwrap();
    }
}
