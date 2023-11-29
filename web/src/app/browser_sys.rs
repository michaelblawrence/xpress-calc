use js_sys::{Function, Promise};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);
}

pub fn timeout(ms: u32) -> timer::TimeoutTimer {
    timer::TimeoutTimer::new(ms)
}

pub fn vibrate(duration_ms: u32) {
    let navigator = sys::navigator();
    navigator.vibrate_with_duration(duration_ms);
}

pub fn show_virtual_kb() -> Result<(), String> {
    let navigator = sys::navigator();
    let virtual_kb: JsValue = sys::property(&navigator, "virtualKeyboard")?;
    let show: Function = sys::property(&virtual_kb, "show")?;
    sys::call(&show, &virtual_kb)?;
    Ok(())
}

pub fn paste_clipboard(
    f: impl wasm_bindgen::closure::IntoWasmClosure<dyn FnMut(JsValue)> + 'static,
) -> Result<(), String> {
    let navigator = sys::navigator();
    let clipboard: JsValue = sys::property(&navigator, "clipboard")?;
    let read_text: Function = sys::property(&clipboard, "readText")?;
    sys::call_promise(&read_text, &clipboard, f)
}

pub mod timer {
    use std::{cell::RefCell, rc::Rc};

    use js_sys::Function;
    use wasm_bindgen::{prelude::Closure, JsCast, JsValue};

    pub struct TimeoutTimer(i32, TimerState<bool>, TimerState<Option<Function>>);

    impl TimeoutTimer {
        pub fn new(ms: u32) -> Self {
            let state: TimerState<bool> = TimerState::default();
            let callback: TimerState<Option<Function>> = TimerState::default();

            let state_clone = state.clone();
            let callback_clone = callback.clone();
            let closure = Closure::new(move |_: JsValue| {
                state_clone.set_value(true);
                if let Some(callback) = callback_clone.value() {
                    if let Err(err) = callback.call0(&JsValue::NULL) {
                        super::log(&format!(
                            "ERROR: failed to run time callback: {}",
                            err.as_string().as_deref().unwrap_or("<unknown>")
                        ))
                    };
                }
            });

            let handle = super::sys::set_timeout(closure, ms);
            Self(handle, state, callback)
        }
        pub fn cancel(&self) {
            if self.expired() {
                return;
            }
            super::sys::clear_timeout(self.handle());
        }
        pub fn handle(&self) -> i32 {
            self.0
        }
        pub fn expired(&self) -> bool {
            self.1.value()
        }
        pub fn with_callback(self, f: impl FnMut() + 'static) -> Self {
            let closure = Closure::<dyn FnMut()>::new(f);
            let closure = closure.into_js_value();
            let handler: Function = closure.dyn_into().unwrap();
            self.2.set_value(Some(handler));
            self
        }
    }

    impl PartialEq for TimeoutTimer {
        fn eq(&self, other: &Self) -> bool {
            self.0 == other.0 && self.1.value() == other.1.value()
        }
    }

    #[derive(Default, Clone)]
    pub struct TimerState<T>(Rc<RefCell<T>>);

    impl<T: Clone> TimerState<T> {
        pub fn set_value(&self, value: T) {
            *self.0.borrow_mut() = value;
        }
        pub fn value(&self) -> T {
            self.0.borrow().clone()
        }
    }
}

mod sys {
    use super::*;

    pub fn navigator() -> web_sys::Navigator {
        web_sys::window().unwrap().navigator()
    }
    pub fn property<T: wasm_bindgen::JsCast>(
        target: &JsValue,
        key: &'static str,
    ) -> Result<T, String> {
        let prop = js_sys::Reflect::get(target, &JsValue::from(key))
            .map_err(|_| format!("missing '{key}'"))?;

        prop.dyn_into()
            .map_err(|_| format!("unable to cast '{key}'"))
    }
    pub fn call(function: &Function, this: &JsValue) -> Result<JsValue, String> {
        function
            .call0(&this)
            .map_err(|_| String::from("error while calling function"))
    }
    pub fn call_promise(
        function: &Function,
        this: &JsValue,
        f: impl wasm_bindgen::closure::IntoWasmClosure<dyn FnMut(JsValue)> + 'static,
    ) -> Result<(), String> {
        let result = function
            .call0(this)
            .map_err(|_| String::from("error while calling async function"))?;
        let promise: &Promise = result
            .dyn_ref()
            .ok_or_else(|| String::from("unable to cast result of function to promise"))?;

        let closure = Closure::new(f);
        _ = promise.then(&closure);
        closure.forget();
        Ok(())
    }
    pub fn set_timeout(handler: Closure<dyn FnMut(JsValue)>, ms: u32) -> i32 {
        let handler = handler.into_js_value();
        let callback: &Function = handler.dyn_ref().unwrap();

        web_sys::window()
            .unwrap()
            .set_timeout_with_callback_and_timeout_and_arguments_0(callback, ms as i32)
            .unwrap()
    }
    pub fn clear_timeout(handle: i32) {
        web_sys::window().unwrap().clear_timeout_with_handle(handle)
    }
}

mod macros {
    #[macro_export]
    macro_rules! console_log {
        ($($arg:tt)*) => {{
            let res = format!($($arg)*);
            crate::app::log(&res);
        }}
    }
}
