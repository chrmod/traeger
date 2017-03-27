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

fn main() {

    let (tx, rx) = mpsc::channel();
    let sender = tx.clone();

    // thread reads from stdin and send messages to main thread
    thread::spawn(move || {
        loop {
            let message = match protocol::read_stdin() {
                Ok(m) => m,
                Err(_) => panic!("nothing more to read"),
            };
            sender.send(message).unwrap();
        }
    });

    let rt = Runtime::new();
    let cx = rt.cx();

    unsafe {
        rooted!(in(cx) let global =
            JS_NewGlobalObject(cx, &SIMPLE_GLOBAL_CLASS, ptr::null_mut(),
                               OnNewGlobalHookOption::FireOnNewGlobalHook,
                               &CompartmentOptions::default())
        );

        loop {
            let message = rx.recv().unwrap();

            rooted!(in(cx) let mut rval = UndefinedValue());

            let wrapped_message = "'".to_string() + message.as_str() + "'";

            rt.evaluate_script(global.handle(), wrapped_message.as_str(),
                "incomming-messsage", 1, rval.handle_mut());

            if (rval.is_string()) {
                let js_string = rval.to_string();
                let response = latin1_to_string(cx, js_string);

                protocol::write_stdout(response);
            }
        }
    }
}
