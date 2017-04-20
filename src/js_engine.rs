use js::jsapi::JS_NewGlobalObject;
use js::jsapi::CompartmentOptions;
use js::jsapi::JSAutoCompartment;
use js::jsapi::OnNewGlobalHookOption;
use js::jsapi::JSContext;
use js::jsapi::Value;
use js::jsapi::CallArgs;
use js::jsapi::JS_DefineFunction;
use js::jsapi::JSObject;
use js::jsapi::Rooted;
use js::jsval::UndefinedValue;
use js::rust::ToString;
use js::rust::Runtime;
use js::rust::SIMPLE_GLOBAL_CLASS;
use js::rust::RootedGuard;
use js::conversions::latin1_to_string;
use std::ptr;

use std::io::Write;

use libc;

pub struct JsEngine {
    rt: Runtime,
    cx: *mut JSContext,
    global: *mut JSObject,
}

impl JsEngine {
    pub fn new() -> JsEngine {
        let rt = Runtime::new().unwrap();
        let cx = rt.cx();
        let h_option = OnNewGlobalHookOption::FireOnNewGlobalHook;
        let c_option = CompartmentOptions::default();
        let script = include_str!("../static/app.js");

        let global = unsafe { JS_NewGlobalObject(cx, &SIMPLE_GLOBAL_CLASS,
                                                 ptr::null_mut(), h_option,
                                                 &c_option)
        };
        let mut __root = Rooted::new_unrooted();
        let global_root = RootedGuard::new(cx, &mut __root, global);

        let _ac = JSAutoCompartment::new(cx, global_root.handle().get());

        unsafe {
            JS_DefineFunction(cx, global_root.handle(),
                b"log\0".as_ptr() as *const libc::c_char,
                Some(log), 1, 0);
        }

        rooted!(in(cx) let mut rval = UndefinedValue());

        rt.evaluate_script(global_root.handle(), script,
        "static/app.js", 0, rval.handle_mut()).unwrap();

        JsEngine {
            rt: rt,
            cx: cx,
            global: global,
        }
    }

    pub fn post_message(&mut self, request: String) -> String {
        let wrapped_message = "onmessage('".to_string() + request.as_str() + "');";
        rooted!(in(self.cx) let global_root = self.global);
        rooted!(in(self.cx) let mut rval = UndefinedValue());

        self.rt.evaluate_script(global_root.handle(), wrapped_message.as_str(),
                                "incomming-messsage", 1, rval.handle_mut()).unwrap();

        if rval.is_string() {
            let js_string = rval.to_string();
            let response = unsafe { latin1_to_string(self.cx, js_string) };
            return response;
        } else {
            return "".to_string();
        }
    }
}

unsafe extern "C" fn log(cx: *mut JSContext, argc: u32, vp: *mut Value) -> bool {
    let args = CallArgs::from_vp(vp, argc);
    let arg = args.get(0);
    let js_string = ToString(cx, arg);
    let string = latin1_to_string(cx, js_string);
    println_stderr!("{}", string);
    return true;
}
