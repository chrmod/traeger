use std::sync::mpsc;
use std::sync::mpsc::{Sender};
use rustc_serialize::json::Json;
use rustc_serialize::json::ToJson;

pub type JSSender = Sender<(Sender<String>, String)>;

pub fn js_message(action: String, args: String) -> String {
    return "{ \"action\": \"".to_string()
        + action.as_str()
        + "\", \"args\": ["
        + args.as_str()
        + "]}";
}

#[allow(unused_must_use)]
pub fn send_async(sender: JSSender, message: String) {
    let (tx, rx) = mpsc::channel();
    sender.send((tx, message));
    rx.recv().unwrap();
}

#[allow(unused_must_use)]
pub fn send_sync(sender: JSSender, message: String) -> String {
    let (tx, rx) = mpsc::channel();
    sender.send((tx, message));
    let rval = rx.recv().unwrap();
    // need to unpack the returned value to get response object
    let data = Json::from_str(rval.as_str()).unwrap();
    let obj = data.as_object().unwrap();
    let response = obj.get("response").unwrap();
    return response.to_json().to_string();
}
