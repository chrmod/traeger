#![feature(lookup_host)]

extern crate rustc_serialize;
extern crate libc;

#[macro_use(println_stderr)]
extern crate webextension_protocol as protocol;
use std::io::Write;

#[macro_use]
extern crate js;

use std::thread;
use std::sync::mpsc;

use std::net::{TcpListener};

use server::SocksServer;
use js_engine::JsEngine;

mod js_engine;
mod server;
mod helpers;

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
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        match listener.local_addr() {
            Ok(socket_name) => {
                let port = socket_name.port();
                let message = helpers::js_message("setPort".to_string(), port.to_string());
                helpers::send_async(sender_for_http.clone(), message);
                println_stderr!("Listening on: {}", socket_name);
            },
            Err(_) => panic!("cannot aquire port"),
        }

        loop {
            match listener.accept() {
                Err(e) => {
                    println_stderr!("There was an error omg {}", e)
                }
                Ok((stream, _)) => {
                    let sender_for_http = sender_for_http.clone();
                    thread::spawn(move || {
                        SocksServer::new(stream, sender_for_http);
                    });
                }
            }
        }
    });

    let mut js_engine = JsEngine::new();

    loop {
        let (io_sender, message) = rx.recv().unwrap();
        let response = js_engine.post_message(message);
        println_stderr!("js returned: {}", response);
        io_sender.send(response).unwrap();
    }
}
