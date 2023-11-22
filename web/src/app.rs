use std::cell::RefCell;

use wasm_bindgen::prelude::*;
use yew::prelude::*;

use xpress_calc::vm::VM;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[function_component(App)]
pub fn app() -> Html {
    let expression = use_state(|| String::from(""));
    let result = use_state(|| None);
    let shift_mode = use_state_eq(|| false);
    let invalid_state = use_state_eq(|| false);
    let vm = use_state(|| RefCell::new(VM::new()));

    use_effect_with(expression.clone(), {
        let result = result.clone();
        let vm = vm.clone();
        let invalid_state = invalid_state.clone();
        let ok_state = {
            let result = result.clone();
            let invalid_state = invalid_state.clone();
            move |x: f64| {
                log(&format!("<computed>: {x}"));
                result.set(Some(x));
                invalid_state.set(false);
            }
        };
        let lazy_state = {
            let invalid_state = invalid_state.clone();
            move |msg: &str| {
                log(msg);
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
                    if program.iter().any(|x| x.has_side_effects()) {
                        lazy_state(
                            "expression may have side-effects, skipping immediate evaluation...",
                        );
                        result.set(None);
                        return;
                    }

                    let mut vm = vm.borrow_mut();
                    match vm.run(&program).clone() {
                        Ok(()) => match (vm.peek_routine().map(|_| ()), vm.pop_result()) {
                            (None, Some(x)) => ok_state(x),
                            (Some(_), _) => err_state("<nan-value>: function"),
                            (None, None) => err_state("<missing-value>: undefined"),
                        },
                        Err(msg) => err_state(&format!("<failed-evaluation>: [{msg}]")),
                    }
                }
                Err(msg) => err_state(&format!("<failed-compilation>: [{msg}]")),
            }
        }
    });

    let onclick = Callback::from({
        let expression = expression.clone();
        let invalid_state = invalid_state.clone();
        let shift_mode = shift_mode.clone();
        let vm = vm.clone();
        move |x: MouseEvent| {
            let target = x.target().unwrap();
            let elem: &web_sys::Element = target.dyn_ref().unwrap();
            let text = elem.text_content().unwrap();
            let c = text.chars().last().unwrap();
            log(&format!("clicked {text}"));

            if c == '⌫' {
                if let Some((end, _)) = expression
                    .char_indices()
                    .rev()
                    .skip_while(|(i, c)| *i == 0 || c.is_whitespace())
                    .next()
                {
                    expression.set(expression[..end].to_string());
                } else {
                    expression.set(String::new());
                }
            } else if c == '⇪' {
                shift_mode.set(!*shift_mode);
            } else if text.as_str() == "AC" {
                expression.set(String::new());
            } else if text.as_str() == "CALC" {
                if !*invalid_state {
                    let result = xpress_calc::compute(&mut vm.borrow_mut(), &*expression);
                    let next_expression = result.map_or_else(|| String::new(), |x| x.to_string());
                    expression.set(next_expression);
                }
            } else if matches!(c, '√') {
                expression.set(format!("{} sqrt(", &*expression));
            } else if c.is_ascii_digit() || matches!(text.as_str(), "c" | "(" | ")") {
                expression.set(format!("{}{}", &*expression, text));
            } else {
                expression.set(format!("{} {} ", &*expression, text));
            }
        }
    });

    let expression = &*expression;
    let result = &*result;
    let onclick_clone = onclick.clone();
    let mini_btn = move |label: &str| {
        html! {
            <div onclick={onclick_clone.clone()} class={classes!("flex-1","px-2","py-6","justify-center","flex","items-center","text-white","text-2xl","font-semibold")}>
                <div class={classes!("rounded-full","h-12","w-12","flex","items-center","bg-cyan-800","justify-center","shadow-lg","border-2","border-cyan-700","hover:border-2","hover:border-gray-500","focus:outline-none")}>{label}</div>
            </div>
        }
    };
    let mini_btn_ref = &mini_btn;
    let shift_mode_clone = shift_mode.clone();
    let mini_btn_dual = move |normal_label: &str, shift_label: &str| {
        if *shift_mode_clone {
            mini_btn_ref(shift_label)
        } else {
            mini_btn_ref(normal_label)
        }
    };
    let onclick_clone = onclick.clone();
    let main_btn = move |label: &str| {
        html! {
            <div onclick={onclick_clone.clone()} class={classes!("flex-1","px-2","py-2","justify-center","flex","items-center","text-white","text-2xl","font-semibold")}>
                <div class={classes!("rounded-full","h-20","w-20","flex","items-center","bg-cyan-800","justify-center","shadow-lg","border-2","border-cyan-700","hover:border-2","hover:border-gray-500","focus:outline-none")}>{label}</div>
            </div>
        }
    };
    html! {
        <div class={classes!("mx-auto","overflow-hidden","mt-2","shadow-lg","mb-2","bg-cyan-900","select-none","shadow-lg","border","border-cyan-700","rounded-lg","lg:w-2/6","md:w-3/6","sm:w-4/6")}>
            <div>
            <div class={classes!("p-5","text-white","text-center","text-3xl","bg-cyan-900")}><span class={classes!("text-blue-500")}>{"XPRESS"}</span>{"CALC"}</div>
            <div class={classes!("pt-12","p-5","pb-0","h-20","select-text","text-white","text-right","text-3xl","bg-cyan-800")}>{ expression }</div>
            <div class={classes!("p-4","h-16","select-text","text-white","text-right","text-3xl","bg-cyan-800")}>
            <div class={classes!("ph-2", "bg-cyan-800")}>
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


        <div class={classes!("flex","items-stretch","bg-cyan-900","h-16")}>
            {mini_btn("⇒")}
            {mini_btn_dual("𝒂", "f")}
            {mini_btn_dual("𝒃", "g")}
            {mini_btn("√")}
            {mini_btn("𝜋")}
            {mini_btn("⇪")}
        </div>

        <div class={classes!("flex","items-stretch","bg-cyan-900","h-16")}>
            {mini_btn("let")}
            {mini_btn_dual("𝒙", "i")}
            {mini_btn_dual("𝒚", "j")}
            {mini_btn_dual("<", "{")}
            {mini_btn_dual(">", "}")}
            {mini_btn_dual("=", ";")}
        </div>

        <div class={classes!("flex","items-stretch","bg-cyan-900","h-24")}>
            {main_btn("AC")}
            {main_btn("(")}
            {main_btn(")")}
            {main_btn("÷")}
        </div>

        <div class={classes!("flex","items-stretch","bg-cyan-900","h-24")}>
            {main_btn("7")}
            {main_btn("8")}
            {main_btn("9")}
            {main_btn("×")}
        </div>

        <div class={classes!("flex","items-stretch","bg-cyan-900","h-24")}>
            {main_btn("4")}
            {main_btn("5")}
            {main_btn("6")}
            {main_btn("-")}
        </div>

        <div class={classes!("flex","items-stretch","bg-cyan-900","h-24")}>
            {main_btn("1")}
            {main_btn("2")}
            {main_btn("3")}
            {main_btn("+")}
        </div>


        <div class={classes!("flex","items-stretch","bg-cyan-900","h-24","mb-4")}>
            {main_btn("0")}
            {main_btn(".")}
            {main_btn("⌫")}
            <div onclick={onclick.clone()} class={classes!("flex-1","px-2","py-2","justify-center","flex","items-center","text-white","text-2xl","font-semibold")}>
            {
                if *invalid_state {
                    html! {
                        <div class={classes!("rounded-full","h-20","w-20","flex","items-center","bg-cyan-900","justify-center","shadow-lg","border-2","border-cyan-800","text-cyan-700","hover:border-2","hover:border-gray-500","focus:outline-none")}>{"CALC"}</div>
                    }
                } else {
                    html! {
                        <div class={classes!("rounded-full","h-20","w-20","flex","items-center","bg-blue-500","justify-center","shadow-lg","border-2","border-cyan-700","hover:border-2","hover:border-gray-500","focus:outline-none")}>{"CALC"}</div>
                    }
                }
            }
            </div>
        </div>


        </div>
        </div>
    }
}
