use wasm_bindgen::prelude::*;
use web_sys::HtmlTextAreaElement;
use yew::prelude::*;

use crate::console_log;

#[derive(Properties, PartialEq)]
pub struct HamburgerMenuProps {
    pub expression: String,
    pub on_open_changed: Callback<bool>,
    #[prop_or_default]
    pub on_expression_changed: Option<Callback<String>>,
}

#[function_component(HamburgerMenu)]
pub fn menu(props: &HamburgerMenuProps) -> Html {
    let opened = use_state(|| false);
    let editor_mode = use_state_eq(|| false);
    let text = use_state(|| String::new());
    let editor_expression = use_state(|| Option::<String>::None);

    let text_clone = text.clone();
    let editor_mode_clone = editor_mode.clone();
    let editor_expression_clone = editor_expression.clone();
    let expression = props.expression.clone();
    let on_expression_changed = props.on_expression_changed.clone();
    use_effect_with(opened.clone(), move |opened| {
        let opened = **opened;
        if !opened {
            editor_mode_clone.set(false);
            let editor_expression = editor_expression_clone
                .as_ref()
                .and_then(|x| xpress_calc::format(&x).ok());

            if let (Some(editor_expression), Some(on_expression_changed)) =
                (editor_expression, on_expression_changed)
            {
                if &editor_expression != &expression {
                    on_expression_changed.emit(editor_expression.clone());
                }
            }
            return;
        }

        match xpress_calc::format_pretty(&expression) {
            Ok(formatted) => text_clone.set(formatted),
            _ => text_clone.set(String::from("<<invalid input>>")),
        };
    });

    let on_open_changed = props.on_open_changed.clone();
    let opened_clone = opened.clone();
    let onclick = Callback::from(move |_: MouseEvent| {
        let next_value = !*opened_clone;
        opened_clone.set(next_value);
        on_open_changed.emit(next_value);
    });

    let text_clone = text.clone();
    let oninput = Callback::from(move |input_event: InputEvent| {
        let event: Event = input_event.dyn_into().unwrap_throw();
        let event_target = event.target().unwrap_throw();
        let target: HtmlTextAreaElement = event_target.dyn_into().unwrap_throw();
        let value = target.value();
        text_clone.set(value.clone());
        if let Ok(_) = xpress_calc::tokenize(&value) {
            editor_expression.set(Some(value));
        } else {
            editor_expression.set(None);
        }
    });

    let editor_mode_clone = editor_mode.clone();
    let btngrp_onclick = Callback::from(move |x: MouseEvent| {
        let target = x.target().unwrap();
        let elem: &web_sys::Element = target.dyn_ref().unwrap();
        let btn_text = elem.text_content().unwrap();
        console_log!("clicked {btn_text}");

        match btn_text.as_str() {
            "Commands" => {
                web_sys::window()
                    .unwrap()
                    .alert_with_message("commands feature is coming soon")
                    .unwrap();
            }
            "History" => {
                web_sys::window()
                    .unwrap()
                    .alert_with_message("history feature is coming soon")
                    .unwrap();
            }
            "Editor" => editor_mode_clone.set(true),
            _ => unreachable!(),
        }
    });

    html! {
        <div>
        <div class="absolute right-4 top-5 z-30">
            <button class="relative group" {onclick}>
                <div class="relative flex overflow-hidden items-center justify-center rounded-full w-[50px] h-[50px] transform transition-all bg-slate-700 ring-0 ring-gray-300 hover:ring-8 group-focus:ring-4 ring-opacity-30 duration-200 shadow-md">
                <div class="flex flex-col justify-between w-[20px] h-[20px] transform transition-all duration-300 origin-center overflow-hidden">
                    <div class="bg-white h-[2px] w-7 transform transition-all duration-300 origin-left group-focus:translate-y-6 delay-100"></div>
                    <div class="bg-white h-[2px] w-7 rounded transform transition-all duration-300 group-focus:translate-y-6 delay-75"></div>
                    <div class="bg-white h-[2px] w-7 transform transition-all duration-300 origin-left group-focus:translate-y-6"></div>

                    <div class="absolute items-center justify-between transform transition-all duration-500 top-2.5 -translate-x-10 group-focus:translate-x-0 flex w-0 group-focus:w-12">
                    <div class="absolute bg-white h-[2px] w-5 transform transition-all duration-500 rotate-0 delay-300 group-focus:rotate-45"></div>
                    <div class="absolute bg-white h-[2px] w-5 transform transition-all duration-500 -rotate-0 delay-300 group-focus:-rotate-45"></div>
                    </div>
                </div>
                </div>
            </button>
        </div>
        if *opened {
            <div class="absolute left-0 top-0 h-screen w-screen bg-gray-800 x-20">
            if !*editor_mode {
                <div class="bg-slate-300 flex items-center justify-center pb-4 pt-24">

                    <div class="inline-flex rounded-md shadow-sm" role="group">
                        <button type="button" onclick={btngrp_onclick.clone()}
                            class="inline-flex items-center px-4 py-2 text-sm font-medium text-gray-900 bg-transparent border border-gray-900 rounded-s-lg hover:bg-gray-900 hover:text-white focus:z-10 focus:ring-2 focus:ring-gray-500 focus:bg-gray-900 focus:text-white">
                            <svg class="w-3 h-3 me-2" aria-hidden="true" xmlns="http://www.w3.org/2000/svg" fill="currentColor"
                                viewBox="0 0 24 24">
                                <path
                                    d="M17.5 3C15.57 3 14 4.57 14 6.5V8h-4V6.5C10 4.57 8.43 3 6.5 3S3 4.57 3 6.5 4.57 10 6.5 10H8v4H6.5C4.57 14 3 15.57 3 17.5S4.57 21 6.5 21s3.5-1.57 3.5-3.5V16h4v1.5c0 1.93 1.57 3.5 3.5 3.5s3.5-1.57 3.5-3.5-1.57-3.5-3.5-3.5H16v-4h1.5c1.93 0 3.5-1.57 3.5-3.5S19.43 3 17.5 3zM16 8V6.5c0-.83.67-1.5 1.5-1.5s1.5.67 1.5 1.5S18.33 8 17.5 8H16zM6.5 8C5.67 8 5 7.33 5 6.5S5.67 5 6.5 5 8 5.67 8 6.5V8H6.5zm3.5 6v-4h4v4h-4zm7.5 5c-.83 0-1.5-.67-1.5-1.5V16h1.5c.83 0 1.5.67 1.5 1.5s-.67 1.5-1.5 1.5zm-11 0c-.83 0-1.5-.67-1.5-1.5S5.67 16 6.5 16H8v1.5c0 .83-.67 1.5-1.5 1.5z" />
                            </svg>
                            {"Commands"}
                        </button>
                        <button type="button" onclick={btngrp_onclick.clone()}
                            class="inline-flex items-center px-4 py-2 text-sm font-medium text-gray-900 bg-transparent border-t border-b border-gray-900 hover:bg-gray-900 hover:text-white focus:z-10 focus:ring-2 focus:ring-gray-500 focus:bg-gray-900 focus:text-white">
                            <svg class="w-3 h-3 me-2" aria-hidden="true" xmlns="http://www.w3.org/2000/svg" fill="currentColor"
                                viewBox="0 0 24 24">
                                <path
                                    d="M13 3c-4.97 0-9 4.03-9 9H1l3.89 3.89.07.14L9 12H6c0-3.87 3.13-7 7-7s7 3.13 7 7-3.13 7-7 7c-1.93 0-3.68-.79-4.94-2.06l-1.42 1.42C8.27 19.99 10.51 21 13 21c4.97 0 9-4.03 9-9s-4.03-9-9-9zm-1 5v5l4.28 2.54.72-1.21-3.5-2.08V8H12z" />
                            </svg>
                            {"History"}
                        </button>
                        <button type="button" onclick={btngrp_onclick.clone()}
                            class="inline-flex items-center px-4 py-2 text-sm font-medium text-gray-900 bg-transparent border border-gray-900 rounded-e-lg hover:bg-gray-900 hover:text-white focus:z-10 focus:ring-2 focus:ring-gray-500 focus:bg-gray-900 focus:text-white">
                            <svg class="w-3 h-3 me-2" aria-hidden="true" xmlns="http://www.w3.org/2000/svg" fill="currentColor"
                            viewBox="0 0 24 24">
                            <path
                                d="M3 17.25V21h3.75L17.81 9.94l-3.75-3.75L3 17.25zM20.71 7.04c.39-.39.39-1.02 0-1.41l-2.34-2.34a.9959.9959 0 0 0-1.41 0l-1.83 1.83 3.75 3.75 1.83-1.83z" />
                            </svg>
                            {"Editor"}
                        </button>
                    </div>
                </div>
            } else {
                <textarea class="text-white text-2xl bg-gray-800 font-normal p-8 h-screen w-screen font-mono"
                    rows="5" cols="33" wrap="off" value={(*text).clone()} {oninput}/>
            }
            </div>
        }
        </div>
    }
}
