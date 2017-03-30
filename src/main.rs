#![feature(lookup_host)]

#[macro_use(println_stderr)]
extern crate webextension_protocol as protocol;
use std::io::Write;

#[macro_use]
extern crate js;
use js::jsapi::JS_NewGlobalObject;
use js::jsapi::CompartmentOptions;
use js::jsapi::OnNewGlobalHookOption;
use js::jsval::UndefinedValue;
use js::rust::Runtime;
use js::rust::SIMPLE_GLOBAL_CLASS;
use js::conversions::latin1_to_string;
use std::ptr;

use std::thread;
use std::sync::mpsc;
use std::fs::File;
use std::io::Read;

use std::net::{TcpListener, TcpStream, Shutdown};

use server::SocksServer;
mod server;

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
        let mut listener = TcpListener::bind("127.0.0.1:1090").unwrap();
        let socket_name = listener.local_addr().unwrap();
        println_stderr!("Listening on: {}", socket_name);

        loop {
            match listener.accept() {
                Err(e) => {
                    println_stderr!("There was an error omg {}", e)
                }
                Ok((stream, remote)) => {
                    thread::spawn(move || {
                        SocksServer::new(stream);
                    });
                }
            }
        }
    });

    let rt = Runtime::new();
    let cx = rt.cx();


    let mut scriptFile = File::open("scripts/concat.js").unwrap();
    let mut script = String::new();
    scriptFile.read_to_string(&mut script);

    unsafe {
        rooted!(in(cx) let global =
            JS_NewGlobalObject(cx, &SIMPLE_GLOBAL_CLASS, ptr::null_mut(),
                               OnNewGlobalHookOption::FireOnNewGlobalHook,
                               &CompartmentOptions::default())
        );

        rooted!(in(cx) let mut rval = UndefinedValue());
        rt.evaluate_script(global.handle(), script.as_str(),
            "scripts/concat.js", 0, rval.handle_mut());

        loop {
            let (io_sender, message) = rx.recv().unwrap();

            rooted!(in(cx) let mut rval = UndefinedValue());

            let wrapped_message = "concat('".to_string() + message.as_str() + "', '');";

            rt.evaluate_script(global.handle(), wrapped_message.as_str(),
                "incomming-messsage", 1, rval.handle_mut());

            if (rval.is_string()) {
                let js_string = rval.to_string();
                let response = latin1_to_string(cx, js_string);

                io_sender.send(response).unwrap();
            }
        }
    }
}
