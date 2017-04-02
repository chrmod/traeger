#![feature(lookup_host)]

extern crate rustc_serialize;

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
use std::env;

use std::net::{TcpListener, TcpStream, Shutdown};

use server::SocksServer;
mod server;
mod helpers;

fn get_listener(ip: String, port: u16, retry: u16) -> Result<TcpListener, ()> {
    let mut p = port;
    let mut r = 0;
    loop {
        match TcpListener::bind((ip.as_str(), 0)) {
            Ok(m) => return Ok(m),
            Err(_) => p += 1,
        }

        r += 1;

        if (r > retry) {
            return Err(());
        }
    }
}

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
        let mut listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
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
                Ok((stream, remote)) => {
                    let sender_for_http = sender_for_http.clone();
                    thread::spawn(move || {
                        SocksServer::new(stream, sender_for_http);
                    });
                }
            }
        }
    });

    let rt = Runtime::new();
    let cx = rt.cx();

    /*
    let current_dir = env::current_dir().unwrap();
    let mut scriptFile = File::open("/home/chrmod/Projects/chrmod/traeger/scripts/filter.js").unwrap();
    let mut script = String::new();
    scriptFile.read_to_string(&mut script);
    */
    let script = include_str!("../scripts/filter.js");

    unsafe {
        rooted!(in(cx) let global =
            JS_NewGlobalObject(cx, &SIMPLE_GLOBAL_CLASS, ptr::null_mut(),
                               OnNewGlobalHookOption::FireOnNewGlobalHook,
                               &CompartmentOptions::default())
        );

        rooted!(in(cx) let mut rval = UndefinedValue());
        rt.evaluate_script(global.handle(), script,
            "scripts/filter.js", 0, rval.handle_mut());

        loop {
            let (io_sender, message) = rx.recv().unwrap();


            rooted!(in(cx) let mut rval = UndefinedValue());

            let wrapped_message = "onmessage('".to_string() + message.as_str() + "');";

            rt.evaluate_script(global.handle(), wrapped_message.as_str(),
                "incomming-messsage", 1, rval.handle_mut());

            if (rval.is_string()) {
                let js_string = rval.to_string();
                let response = latin1_to_string(cx, js_string);

                println_stderr!("js returned: {}", response);
                io_sender.send(response).unwrap();
            }
        }
    }
}
