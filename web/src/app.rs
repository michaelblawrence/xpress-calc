use wasm_bindgen::prelude::*;
use web_sys::HtmlInputElement;
use yew::prelude::*;

use xpress_calc::vm::VM;

use crate::{
    app::{browser_sys::log, event::ButtonEvent},
    console_log,
};

use self::browser_sys::timer::TimeoutTimer;

mod browser_sys;

#[function_component(App)]
pub fn app() -> Html {
    let expression = use_state(|| String::from(""));
    let result = use_state(|| None);
    let shift_mode = use_state_eq(|| false);
    let invalid_state = use_state_eq(|| false);
    let vm = use_mut_ref(|| VM::new());

    #[derive(Default)]
    struct TimerHandle {
        fmt_btn: Option<TimeoutTimer>,
    }
    let timer_handles = use_mut_ref(|| TimerHandle::default());

    use_effect_with(expression.clone(), {
        let result = result.clone();
        let vm = vm.clone();
        let invalid_state = invalid_state.clone();
        let ok_state = {
            let result = result.clone();
            let invalid_state = invalid_state.clone();
            move |x: f64| {
                console_log!("<computed>: {x}");
                result.set(Some(x));
                invalid_state.set(false);
            }
        };
        let err_state = {
            let invalid_state = invalid_state.clone();
            move |msg: &str| {
                log(msg);
                invalid_state.set(true);
            }
        };
        move |expression| {
            if expression.is_empty() {
                invalid_state.set(false);
                result.set(None);
                return;
            }
            match xpress_calc::compile(&*expression) {
                Ok(program) => {
                    let mut vm = vm.borrow().clone();
                    match vm.run(&program).clone() {
                        Ok(()) => match (vm.peek_routine().map(|_| ()), vm.pop_result()) {
                            (None, Some(x)) => ok_state(x),
                            (Some(_), _) => err_state("<nan-value>: function"),
                            (None, None) => {
                                log("<missing-value>: undefined");
                                invalid_state.set(false);
                            }
                        },
                        Err(msg) => err_state(&format!("<failed-evaluation>: [{msg}]")),
                    }
                }
                Err(msg) => err_state(&format!("<failed-compilation>: [{msg}]")),
            }
        }
    });

    let create_button_ctx = {
        let expression = expression.clone();
        let invalid_state = invalid_state.clone();
        let shift_mode = shift_mode.clone();
        let vm = vm.clone();
        move || {
            let expression_val = (*expression).clone();
            let expression = expression.clone();
            let shift_mode = shift_mode.clone();
            let invalid_state = *invalid_state;
            let vm = vm.clone();
            let set_expression = move |x| expression.set(x);
            let toggle_shift = move || shift_mode.set(!*shift_mode);
            event::ButtonEventContext::new(
                expression_val,
                invalid_state,
                vm,
                set_expression,
                toggle_shift,
            )
        }
    };

    let create_button_ctx_clone = create_button_ctx.clone();
    let onclick = Callback::from(move |x: MouseEvent| {
        let target = x.target().unwrap();
        let elem: &web_sys::Element = target.dyn_ref().unwrap();
        let btn_text = elem.text_content().unwrap();
        console_log!("clicked {btn_text}");
        let ctx = create_button_ctx_clone();

        match btn_text.as_str() {
            "⌫" => ButtonEvent::emit(ButtonEvent::Backspace, ctx),
            "⇪" => ButtonEvent::emit(ButtonEvent::Shift, ctx),
            "ABC" => ButtonEvent::emit(ButtonEvent::ShowKeyboard, ctx),
            "📋" => ButtonEvent::emit(ButtonEvent::PasteClipboard, ctx),
            "AC" => ButtonEvent::emit(ButtonEvent::AC, ctx),
            "CALC" => ButtonEvent::emit(ButtonEvent::CALC, ctx),
            "√" => ButtonEvent::emit(ButtonEvent::EmitSqrt, ctx),
            "let" => ButtonEvent::emit(ButtonEvent::EmitLet, ctx),
            "=" => ButtonEvent::emit(ButtonEvent::EmitEqual, ctx),
            "." => ButtonEvent::emit(ButtonEvent::EmitDP, ctx),
            "➪" => ButtonEvent::emit(ButtonEvent::EmitFnArrow, ctx),
            _ => ButtonEvent::emit(ButtonEvent::Emit(btn_text), ctx),
        }
    });

    let onmousedown = Callback::from(move |_: MouseEvent| browser_sys::vibrate(40));

    let expression_clone = expression.clone();
    let timer_handles_clone = timer_handles.clone();
    let fmt_btn_oncursordown = move || {
        let expression = expression_clone.clone();
        let handle = browser_sys::timeout(200).with_callback(move || {
            console_log!("fmt_btn held down");
            match xpress_calc::format(&*expression) {
                Ok(formatted) => {
                    if formatted.as_str() != expression.as_str() {
                        log("using pretty printed expression");
                        expression.set(formatted);
                        browser_sys::vibrate(40);
                    }
                }
                Err(e) => console_log!("unable to pretty print expression: {e}"),
            }
        });
        (*timer_handles_clone).borrow_mut().fmt_btn = Some(handle);
    };
    let timer_handles_clone = timer_handles.clone();
    let fmt_btn_oncursorup = move || {
        if let Some(handle) = &timer_handles_clone.borrow().fmt_btn {
            handle.cancel();
        }
    };

    let fmt_btn_onmousedown = Callback::from({
        let fmt_btn_oncursordown = fmt_btn_oncursordown.clone();
        move |_: MouseEvent| fmt_btn_oncursordown()
    });
    let fmt_btn_onmouseup = Callback::from({
        let fmt_btn_oncursorup = fmt_btn_oncursorup.clone();
        move |_: MouseEvent| fmt_btn_oncursorup()
    });
    let fmt_btn_ontouchstart = Callback::from({
        let fmt_btn_oncursordown = fmt_btn_oncursordown.clone();
        move |_: TouchEvent| fmt_btn_oncursordown()
    });
    let fmt_btn_ontouchend = Callback::from({
        let fmt_btn_oncursorup = fmt_btn_oncursorup.clone();
        move |_: TouchEvent| fmt_btn_oncursorup()
    });

    let expression_clone = expression.clone();
    let oninput = Callback::from(move |input_event: InputEvent| {
        let event: Event = input_event.dyn_into().unwrap_throw();
        let event_target = event.target().unwrap_throw();
        let target: HtmlInputElement = event_target.dyn_into().unwrap_throw();
        let value = target.value();
        expression_clone.set(value);
    });

    let invalid_state_clone = invalid_state.clone();
    let onkeypress = Callback::from(move |kb_event: KeyboardEvent| {
        const ENTER_KEY: u32 = 13;

        match kb_event.char_code() {
            ENTER_KEY if !*invalid_state_clone => {
                let ctx = create_button_ctx();
                ButtonEvent::emit(ButtonEvent::CALC, ctx);
            }
            _ => {}
        }
    });

    let expression = &*expression;
    let result = &*result;
    let onclick_clone = onclick.clone();
    let onmousedown_clone = onmousedown.clone();
    let mini_btn = move |ButtonProp { label, theme }| {
        let theme = theme.unwrap_or("bg-gray-800");
        html! {
            <div onclick={onclick_clone.clone()} onmousedown={onmousedown_clone.clone()} class={classes!("flex-1","px-2","py-6","justify-center","flex","items-center","text-white","text-2xl","font-semibold")}>
                <div class={classes!("rounded-full","h-12","w-12","flex","items-center",theme,"justify-center","shadow-lg","border-2","border-gray-700","hover:border-2","hover:border-gray-500","focus:outline-none")}>{label}</div>
            </div>
        }
    };
    let mini_btn_ref = &mini_btn;
    let shift_mode_clone = shift_mode.clone();
    let mini_btn_dual = move |normal_label: ButtonProp, shift_label: ButtonProp| {
        if *shift_mode_clone {
            mini_btn_ref(shift_label)
        } else {
            mini_btn_ref(normal_label)
        }
    };
    let onclick_clone = onclick.clone();
    let onmousedown_clone = onmousedown.clone();
    let main_btn = move |label: &str| {
        html! {
            <div onclick={onclick_clone.clone()} onmousedown={onmousedown_clone.clone()} class={classes!("flex-1","px-2","py-2","justify-center","flex","items-center","text-white","text-4xl","font-semibold")}>
                <div class={classes!("rounded-full","h-20","w-20","flex","items-center","bg-gray-800","justify-center","shadow-lg","border-2","border-gray-700","hover:border-2","hover:border-gray-500","focus:outline-none")}>{label}</div>
            </div>
        }
    };
    let main_btn_ref = &main_btn;
    let shift_mode_clone = shift_mode.clone();
    let main_btn_dual = move |normal_label: &str, shift_label: &str| {
        if *shift_mode_clone {
            main_btn_ref(shift_label)
        } else {
            main_btn_ref(normal_label)
        }
    };
    let shift_mode = *shift_mode;

    html! {
        <div class={classes!("mx-auto","overflow-hidden","mt-2","shadow-lg","mb-2","bg-gray-900","select-none","shadow-lg","border","border-gray-700","rounded-lg","lg:w-2/6","md:w-3/6","sm:w-4/6")}>
            <div>
            <div onmousedown={fmt_btn_onmousedown} onmouseup={fmt_btn_onmouseup} ontouchstart={fmt_btn_ontouchstart} ontouchend={fmt_btn_ontouchend}
            class={classes!("p-5","text-white","text-center","text-3xl","bg-gray-900")}>
                <span class={classes!("text-blue-500")}>{"XPRESS"}</span>{"CALC"}
            </div>
            <input
                value={expression.clone()}
                {oninput}
                {onkeypress}
                class={classes!("w-full","border-none","pt-12","p-5","pb-0","h-20","select-text","text-white","text-right","text-3xl","bg-gray-800")}
                />
            <div class={classes!("p-4","h-16","select-text","text-white","text-right","text-3xl","bg-gray-800")}>
            <div class={classes!("ph-2", "bg-gray-800")}>
            {
                if *invalid_state {
                    html!{
                        <span class={classes!("text-blue-300", "blur-sm", "transition", "animate-pulse")}>{ result }</span>
                    }
                } else {
                    html!{
                        <span class={classes!("text-blue-300", "blur-none", "transition", "animate-pulse")}>{ result }</span>
                    }
                }
            }
            </div>
            </div>


        <div class={classes!("flex","items-stretch","bg-gray-900","h-16","mt-4")}>
            {mini_btn_dual("➪".into(), "📋".into())}
            {mini_btn_dual("𝒂".into(), "f".into())}
            {mini_btn_dual("𝒃".into(), "g".into())}
            {mini_btn_dual("if".into(), "else".into())}
            {mini_btn_dual(";".into(), "𝜋".into())}
            {mini_btn(ButtonProp {label: "⇪", theme: shift_mode.then_some("bg-yellow-900")})}
        </div>

        <div class={classes!("flex","items-stretch","bg-gray-900","h-16")}>
            {mini_btn("let".into())}
            {mini_btn_dual("𝒙".into(), "i".into())}
            {mini_btn_dual("𝒚".into(), "j".into())}
            {mini_btn_dual("<".into(), "{".into())}
            {mini_btn_dual(">".into(), "}".into())}
            {mini_btn("=".into())}
        </div>

        <div class={classes!("flex","items-stretch","bg-gray-900","h-24","mt-2")}>
            {main_btn("AC")}
            {main_btn("(")}
            {main_btn(")")}
            {main_btn("÷")}
        </div>

        <div class={classes!("flex","items-stretch","bg-gray-900","h-24")}>
            {main_btn("7")}
            {main_btn("8")}
            {main_btn("9")}
            {main_btn("×")}
        </div>

        <div class={classes!("flex","items-stretch","bg-gray-900","h-24")}>
            {main_btn("4")}
            {main_btn("5")}
            {main_btn("6")}
            {main_btn("−")}
        </div>

        <div class={classes!("flex","items-stretch","bg-gray-900","h-24")}>
            {main_btn("1")}
            {main_btn("2")}
            {main_btn("3")}
            {main_btn("+")}
        </div>


        <div class={classes!("flex","items-stretch","bg-gray-900","h-24","mb-4")}>
            {main_btn("0")}
            {main_btn_dual(".", "ABC")}
            {main_btn("⌫")}
            <div onclick={onclick.clone()} onmousedown={onmousedown.clone()} class={classes!("flex-1","px-2","py-2","justify-center","flex","items-center","text-white","text-2xl","font-semibold")}>
            {
                if *invalid_state {
                    html! {
                        <div class={classes!("rounded-full","h-20","w-20","flex","items-center","bg-gray-900","justify-center","shadow-lg","border-2","border-gray-800","text-gray-700","hover:border-2","hover:border-gray-500","focus:outline-none")}>{"CALC"}</div>
                    }
                } else {
                    html! {
                        <div class={classes!("rounded-full","h-20","w-20","flex","items-center","bg-blue-500","justify-center","shadow-lg","border-2","border-gray-700","hover:border-2","hover:border-gray-500","focus:outline-none")}>{"CALC"}</div>
                    }
                }
            }
            </div>
        </div>


        </div>
        </div>
    }
}

struct ButtonProp {
    label: &'static str,
    theme: Option<&'static str>,
}

impl From<&'static str> for ButtonProp {
    fn from(label: &'static str) -> Self {
        Self { label, theme: None }
    }
}

impl From<(&'static str, &'static str)> for ButtonProp {
    fn from((label, theme): (&'static str, &'static str)) -> Self {
        Self {
            label,
            theme: Some(theme),
        }
    }
}

mod event {
    use std::{cell::RefCell, rc::Rc};

    use wasm_bindgen::JsValue;
    use xpress_calc::vm::{Instruction, VM};

    use crate::console_log;

    use super::browser_sys;

    pub enum ButtonEvent {
        Backspace,
        Shift,
        ShowKeyboard,
        PasteClipboard,
        AC,
        CALC,
        EmitSqrt,
        EmitLet,
        EmitEqual,
        EmitDP,
        EmitFnArrow,
        Emit(String),
    }

    pub struct ButtonEventContext {
        expression: String,
        invalid_state: bool,
        vm: Rc<RefCell<VM>>,
        boxed_set_expression: Box<dyn Fn(String) + 'static>,
        boxed_toggle_shift: Box<dyn Fn() + 'static>,
    }

    impl ButtonEventContext {
        pub fn new(
            expression: String,
            invalid_state: bool,
            vm: Rc<RefCell<VM>>,
            set_expression: impl Fn(String) + 'static,
            toggle_shift: impl Fn() + 'static,
        ) -> Self {
            Self {
                expression,
                invalid_state,
                vm,
                boxed_set_expression: Box::new(set_expression),
                boxed_toggle_shift: Box::new(toggle_shift),
            }
        }
        pub fn append_expression(&self, x: &str) {
            (self.boxed_set_expression)(format!("{}{}", self.expression, x))
        }
        pub fn set_expression(&self, x: String) {
            (self.boxed_set_expression)(x)
        }
        pub fn toggle_shift(&self) {
            (self.boxed_toggle_shift)()
        }
    }

    impl ButtonEvent {
        pub fn emit(event: ButtonEvent, ctx: ButtonEventContext) {
            match event {
                ButtonEvent::Backspace => {
                    if let Some((end, _)) = ctx
                        .expression
                        .char_indices()
                        .rev()
                        .skip_while(|(i, c)| *i == 0 || c.is_whitespace())
                        .next()
                    {
                        ctx.set_expression(ctx.expression[..end].to_string());
                    } else {
                        ctx.set_expression(String::new());
                    }
                }
                ButtonEvent::Shift => {
                    ctx.toggle_shift();
                }
                ButtonEvent::ShowKeyboard => {
                    if let Err(e) = browser_sys::show_virtual_kb() {
                        console_log!("ERROR on show_virtual_kb: {e}")
                    }
                }
                ButtonEvent::PasteClipboard => {
                    let expression = ctx.expression.to_owned();
                    let on_paste = move |s: JsValue| {
                        ctx.set_expression(format!("{}{}", expression, s.as_string().unwrap()))
                    };
                    if let Err(e) = browser_sys::paste_clipboard(on_paste) {
                        console_log!("ERROR on paste_clipboard: {e}")
                    }
                }
                ButtonEvent::AC => {
                    ctx.set_expression(String::new());
                }
                ButtonEvent::CALC => {
                    if ctx.invalid_state {
                        return;
                    }

                    let vm: &mut VM = &mut (*ctx.vm).borrow_mut();
                    let expression: &str = &ctx.expression;
                    let mut ident = None;

                    let result = xpress_calc::compile(&*expression)
                        .and_then(|program| {
                            if let Some(Instruction::Assign(set)) = program.last() {
                                ident = Some(set.clone());
                            }
                            vm.run(&program)
                        })
                        .and_then(|_| vm.pop_result().ok_or_else(|| String::from("no result")));

                    match result {
                        Ok(x) => ctx.set_expression(x.to_string()),
                        Err(err) => {
                            console_log!("ERROR: {err}");
                            ctx.set_expression(ident.unwrap_or_default())
                        }
                    }
                }
                ButtonEvent::EmitSqrt => {
                    ctx.append_expression("sqrt(");
                }
                ButtonEvent::EmitLet => {
                    ctx.append_expression("let ");
                }
                ButtonEvent::EmitEqual => {
                    ctx.append_expression(" = ");
                }
                ButtonEvent::EmitDP => {
                    if !ctx.expression.ends_with('.') {
                        ctx.append_expression(".");
                    }
                }
                ButtonEvent::EmitFnArrow => {
                    ctx.append_expression(" ➪ ");
                }
                ButtonEvent::Emit(text) => {
                    ctx.append_expression(&text);
                }
            }
        }
    }
}
