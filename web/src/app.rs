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
    let expression = use_state(|| String::from("2000 + 100"));
    let result = use_state(|| Some(2100.0));
    let vm = use_state(|| RefCell::new(VM::new()));

    use_effect_with(expression.clone(), {
        let result = result.clone();
        move |expression| match xpress_calc::compute(&mut vm.borrow_mut(), &*expression) {
            Some(x) => {
                log(&format!("{x}"));
                result.set(Some(x));
            }
            None => log("<undefined>"),
        }
    });

    let onclick = Callback::from({
        let expression = expression.clone();
        let result = result.clone();
        move |x: MouseEvent| {
            let target = x.target().unwrap();
            let elem: &web_sys::Element = target.dyn_ref().unwrap();
            let content = elem.text_content().unwrap().chars().next().unwrap();
            log(&format!("clicked {content}"));

            if content == '⌫' {
                if let Some((end, c)) = expression
                    .char_indices()
                    .rev()
                    .skip(1)
                    .find(|(_, c)| c.is_ascii_digit())
                {
                    let end = end + c.len_utf8();
                    expression.set(expression[..end].to_string());
                } else {
                    expression.set(String::new());
                    result.set(None);
                }
            } else if content.is_ascii_digit() || matches!(content, 'c') {
                expression.set(format!("{}{}", &*expression, content));
            } else {
                expression.set(format!("{} {} ", &*expression, content));
            }
        }
    });

    let expression = &*expression;
    let result = &*result;
    html! {
        <div class={classes!("mx-auto","overflow-hidden","mt-10","shadow-lg","mb-2","bg-cyan-900","shadow-lg","border","rounded-lg","lg:w-2/6","md:w-3/6","sm:w-4/6")}>
            <div>
            <div class={classes!("p-5","text-white","text-center","text-3xl","bg-cyan-900")}><span class={classes!("text-blue-500")}>{"XPRESS"}</span>{"CALC"}</div>
            <div class={classes!("pt-16","p-5","pb-0","text-white","text-right","text-3xl","bg-cyan-800")}>{ expression }</div>
            <div class={classes!("p-5","text-white","text-right","text-3xl","bg-cyan-800")}>{"= "}<span class={classes!("text-blue-500")}>{ result }</span></div>


        <div class={classes!("flex","items-stretch","bg-cyan-900","h-24")}>
            <div onclick={onclick.clone()} class={classes!("flex-1","px-2","py-2","justify-center","flex","items-center","text-white","text-2xl","font-semibold")}>
            <div class={classes!("rounded-full","h-20","w-20","flex","items-center","bg-cyan-800","justify-center","shadow-lg","border-2","border-cyan-700","hover:border-2","hover:border-gray-500","focus:outline-none")}>{"⌫"}</div>
            </div>

            <div onclick={onclick.clone()} class={classes!("flex-1","px-2","py-2","justify-center","flex","items-center","text-white","text-2xl","font-semibold")}>
            <div class={classes!("rounded-full","h-20","w-20","flex","items-center","bg-cyan-800","justify-center","shadow-lg","border-2","border-cyan-700","hover:border-2","hover:border-gray-500","focus:outline-none")}>{"("}</div>
            </div>

            <div onclick={onclick.clone()} class={classes!("flex-1","px-2","py-2","justify-center","flex","items-center","text-white","text-2xl","font-semibold")}>
            <div class={classes!("rounded-full","h-20","w-20","flex","items-center","bg-cyan-800","justify-center","shadow-lg","border-2","border-cyan-700","hover:border-2","hover:border-gray-500","focus:outline-none")}>{")"}</div>
            </div>

            <div onclick={onclick.clone()} class={classes!("flex-1","px-2","py-2","justify-center","flex","items-center","text-white","text-2xl","font-semibold")}>
            <div class={classes!("rounded-full","h-20","w-20","flex","items-center","bg-cyan-800","justify-center","shadow-lg","border-2","border-cyan-700","hover:border-2","hover:border-gray-500","focus:outline-none")}>{"÷"}</div>
            </div>
        </div>

        <div class={classes!("flex","items-stretch","bg-cyan-900","h-24")}>
            <div onclick={onclick.clone()} class={classes!("flex-1","px-2","py-2","justify-center","flex","items-center","text-white","text-2xl","font-semibold")}>
            <div class={classes!("rounded-full","h-20","w-20","flex","items-center","bg-cyan-800","justify-center","shadow-lg","border-2","border-cyan-700","hover:border-2","hover:border-gray-500","focus:outline-none")}>{"7"}</div>
            </div>

            <div onclick={onclick.clone()} class={classes!("flex-1","px-2","py-2","justify-center","flex","items-center","text-white","text-2xl","font-semibold")}>
            <div class={classes!("rounded-full","h-20","w-20","flex","items-center","bg-cyan-800","justify-center","shadow-lg","border-2","border-cyan-700","hover:border-2","hover:border-gray-500","focus:outline-none")}>{"8"}</div>
            </div>

            <div onclick={onclick.clone()} class={classes!("flex-1","px-2","py-2","justify-center","flex","items-center","text-white","text-2xl","font-semibold")}>
            <div class={classes!("rounded-full","h-20","w-20","flex","items-center","bg-cyan-800","justify-center","shadow-lg","border-2","border-cyan-700","hover:border-2","hover:border-gray-500","focus:outline-none")}>{"9"}</div>
            </div>

            <div onclick={onclick.clone()} class={classes!("flex-1","px-2","py-2","justify-center","flex","items-center","text-white","text-2xl","font-semibold")}>
            <div class={classes!("rounded-full","h-20","w-20","flex","items-center","bg-cyan-800","justify-center","shadow-lg","border-2","border-cyan-700","hover:border-2","hover:border-gray-500","focus:outline-none")}>{"×"}</div>
            </div>
        </div>

        <div class={classes!("flex","items-stretch","bg-cyan-900","h-24")}>
            <div onclick={onclick.clone()} class={classes!("flex-1","px-2","py-2","justify-center","flex","items-center","text-white","text-2xl","font-semibold")}>
            <div class={classes!("rounded-full","h-20","w-20","flex","items-center","bg-cyan-800","justify-center","shadow-lg","border-2","border-cyan-700","hover:border-2","hover:border-gray-500","focus:outline-none")}>{"4"}</div>
            </div>

            <div onclick={onclick.clone()} class={classes!("flex-1","px-2","py-2","justify-center","flex","items-center","text-white","text-2xl","font-semibold")}>
            <div class={classes!("rounded-full","h-20","w-20","flex","items-center","bg-cyan-800","justify-center","shadow-lg","border-2","border-cyan-700","hover:border-2","hover:border-gray-500","focus:outline-none")}>{"5"}</div>
            </div>

            <div onclick={onclick.clone()} class={classes!("flex-1","px-2","py-2","justify-center","flex","items-center","text-white","text-2xl","font-semibold")}>
            <div class={classes!("rounded-full","h-20","w-20","flex","items-center","bg-cyan-800","justify-center","shadow-lg","border-2","border-cyan-700","hover:border-2","hover:border-gray-500","focus:outline-none")}>{"6"}</div>
            </div>

            <div onclick={onclick.clone()} class={classes!("flex-1","px-2","py-2","justify-center","flex","items-center","text-white","text-2xl","font-semibold")}>
            <div class={classes!("rounded-full","h-20","w-20","flex","items-center","bg-cyan-800","justify-center","shadow-lg","border-2","border-cyan-700","hover:border-2","hover:border-gray-500","focus:outline-none")}>{"-"}</div>
            </div>
        </div>

        <div class={classes!("flex","items-stretch","bg-cyan-900","h-24")}>
            <div onclick={onclick.clone()} class={classes!("flex-1","px-2","py-2","justify-center","flex","items-center","text-white","text-2xl","font-semibold")}>
            <div class={classes!("rounded-full","h-20","w-20","flex","items-center","bg-cyan-800","justify-center","shadow-lg","border-2","border-cyan-700","hover:border-2","hover:border-gray-500","focus:outline-none")}>{"1"}</div>
            </div>

            <div onclick={onclick.clone()} class={classes!("flex-1","px-2","py-2","justify-center","flex","items-center","text-white","text-2xl","font-semibold")}>
            <div class={classes!("rounded-full","h-20","w-20","flex","items-center","bg-cyan-800","justify-center","shadow-lg","border-2","border-cyan-700","hover:border-2","hover:border-gray-500","focus:outline-none")}>{"2"}</div>
            </div>

            <div onclick={onclick.clone()} class={classes!("flex-1","px-2","py-2","justify-center","flex","items-center","text-white","text-2xl","font-semibold")}>
            <div class={classes!("rounded-full","h-20","w-20","flex","items-center","bg-cyan-800","justify-center","shadow-lg","border-2","border-cyan-700","hover:border-2","hover:border-gray-500","focus:outline-none")}>{"3"}</div>
            </div>

            <div onclick={onclick.clone()} class={classes!("flex-1","px-2","py-2","justify-center","flex","items-center","text-white","text-2xl","font-semibold")}>
            <div class={classes!("rounded-full","h-20","w-20","flex","items-center","bg-cyan-800","justify-center","shadow-lg","border-2","border-cyan-700","hover:border-2","hover:border-gray-500","focus:outline-none")}>{"+"}</div>
            </div>
        </div>


        <div class={classes!("flex","items-stretch","bg-cyan-900","h-24","mb-4")}>
            <div onclick={onclick.clone()} class={classes!("flex-1","px-2","py-2","justify-center","flex","items-center","text-white","text-2xl","font-semibold")}>
            <div class={classes!("rounded-full","h-20","w-20","flex","items-center","bg-cyan-800","justify-center","shadow-lg","border-2","border-cyan-700","hover:border-2","hover:border-gray-500","focus:outline-none")}>{"+"}</div>
            </div>

            <div onclick={onclick.clone()} class={classes!("flex-1","px-2","py-2","justify-center","flex","items-center","text-white","text-2xl","font-semibold")}>
            <div class={classes!("rounded-full","h-20","w-20","flex","items-center","bg-cyan-800","justify-center","shadow-lg","border-2","border-cyan-700","hover:border-2","hover:border-gray-500","focus:outline-none")}>{"0"}</div>
            </div>

            <div onclick={onclick.clone()} class={classes!("flex-1","px-2","py-2","justify-center","flex","items-center","text-white","text-2xl","font-semibold")}>
            <div class={classes!("rounded-full","h-20","w-20","flex","items-center","bg-cyan-800","justify-center","shadow-lg","border-2","border-cyan-700","hover:border-2","hover:border-gray-500","focus:outline-none")}>{"."}</div>
            </div>

            <div onclick={onclick.clone()} class={classes!("flex-1","px-2","py-2","justify-center","flex","items-center","text-white","text-2xl","font-semibold")}>
            <div class={classes!("rounded-full","h-20","w-20","flex","items-center","bg-blue-500","justify-center","shadow-lg","border-2","border-cyan-700","hover:border-2","hover:border-gray-500","focus:outline-none")}>{"="}</div>
            </div>
        </div>


        </div>
        </div>
    }
}
