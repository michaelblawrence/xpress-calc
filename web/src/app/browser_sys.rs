use js_sys::{Function, Promise};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);
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
