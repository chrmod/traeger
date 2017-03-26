#[macro_use(println_stderr)]
extern crate webextension_protocol as protocol;
use std::io::Write;
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

    loop {
        let message = rx.recv().unwrap();
        println_stderr!("received {}", message);
        protocol::write_stdout(message);
    }
}
