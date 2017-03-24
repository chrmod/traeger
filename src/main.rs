extern crate webextension_rust_template as protocol;
use std::io;
use std::thread;
use std::sync::mpsc;

fn main() {
    let (tx, rx) = mpsc::channel();
    let sender = tx.clone();

    // thread reads from stdin and send messages to main thread
    thread::spawn(move || {
        loop {
            let f = protocol::Input::Stdin(io::stdin());
            let message = protocol::read(f);
            sender.send(message).unwrap();
        }
    });

    loop {
        let message = rx.recv().unwrap();
        let output = protocol::Output::Stdout(io::stdout());
        protocol::write(output, message.to_string());
    }
}
