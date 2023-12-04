use wasm_bindgen::prelude::*;
use web_sys::{Element, HtmlElement, HtmlTextAreaElement};
use yew::prelude::*;

use crate::console_log;

#[derive(Properties, PartialEq, Clone)]
pub struct HamburgerMenuProps {
    pub expression: String,
    #[prop_or_default]
    pub opened: bool,
    pub on_open_changed: Callback<bool>,
    pub expression_palette: serde_json::Value,
    pub expression_history: serde_json::Value,
    #[prop_or_default]
    pub on_expression_changed: Option<Callback<String>>,
}

#[function_component(HamburgerMenu)]
pub fn menu(props: &HamburgerMenuProps) -> Html {
    let mode = use_state(|| MenuMode::Hidden);

    let mode_clone = mode.clone();
    use_effect_with(props.opened, move |&x| {
        mode_clone.set(if x { MenuMode::None } else { MenuMode::Hidden })
    });

    let on_open_changed = props.on_open_changed.clone();
    let mode_clone = mode.clone();
    let onclick = Callback::from(move |_: MouseEvent| {
        let next_value = mode_clone.toggled();
        mode_clone.set(next_value);
        on_open_changed.emit(next_value.is_open());
    });

    let mode_clone = mode.clone();
    let on_open_changed = props.on_open_changed.clone();
    let on_mode_changed = Callback::from(move |x: MenuMode| {
        let opened_changed = mode_clone.is_open() != x.is_open();
        mode_clone.set(x);
        if opened_changed {
            on_open_changed.emit(x.is_open());
        }
    });

    let props = Some(*mode)
        .filter(|x| x.is_open())
        .map(move |_| props.clone());

    html! {
        <div>
        <div class="absolute right-4 top-5 z-30">
            <HamburgerButton {onclick} focussed={mode.is_open()} />
        </div>
        <HamburgerMenuScreen mode={*mode} {on_mode_changed}>
            if let Some(props) = props {
                <HamburgerMenuDrawer
                    mode={*mode}
                    expression={props.expression}
                    expression_palette={props.expression_palette}
                    expression_history={props.expression_history}
                    on_expression_changed={props.on_expression_changed} />
            }
        </HamburgerMenuScreen>
        </div>
    }
}

#[derive(Properties, PartialEq, Clone)]
struct HamburgerMenuScreenProps {
    pub mode: MenuMode,
    pub on_mode_changed: Callback<MenuMode>,
    pub children: Html,
}

#[function_component(HamburgerMenuScreen)]
fn menu_screen(props: &HamburgerMenuScreenProps) -> Html {
    let on_mode_changed = props.on_mode_changed.clone();
    let screen_onclick = Callback::from(move |e: MouseEvent| {
        if let Some(target_element) = e.target().and_then(|x| x.dyn_into::<Element>().ok()) {
            if target_element.id() == "screen" {
                on_mode_changed.emit(MenuMode::Hidden);
            }
        }
    });

    let on_mode_changed = props.on_mode_changed.clone();
    let btngrp_onclick = Callback::from(move |e: MouseEvent| {
        e.stop_propagation();
        let target = e.target().unwrap();
        let elem: &web_sys::Element = target.dyn_ref().unwrap();
        let btn_text = elem.text_content().unwrap();
        console_log!("clicked {btn_text}");

        match btn_text.as_str() {
            "Commands" => on_mode_changed.emit(MenuMode::Commands),
            "History" => on_mode_changed.emit(MenuMode::History),
            "Editor" => on_mode_changed.emit(MenuMode::Editor),
            _ => unreachable!(),
        }
    });

    html! {
        if let MenuMode::Hidden = props.mode {
            <div id="screen" class={classes!("absolute","left-0","top-0","h-0","w-screen","x-20","transition-all", "bg-gray-950/0")}></div>
        } else {
            <div id="screen" class={classes!("absolute","left-0","top-0","h-screen","w-screen","x-20","transition-all", "bg-gray-950/90")} onclick={screen_onclick}>
            if let MenuMode::None = props.mode {
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
                { props.children.clone() }
            }
            </div>
        }
    }
}

#[derive(Properties, PartialEq, Clone)]
struct HamburgerMenuDrawerProps {
    pub expression: String,
    pub mode: MenuMode,
    pub expression_palette: serde_json::Value,
    pub expression_history: serde_json::Value,
    pub on_expression_changed: Option<Callback<String>>,
}

#[function_component(HamburgerMenuDrawer)]
fn menu_drawer(props: &HamburgerMenuDrawerProps) -> Html {
    let text = use_state(|| String::new());
    let editor_expression = use_state(|| Option::<String>::None);

    let text_clone = text.clone();
    let editor_expression_clone = editor_expression.clone();
    let expression = props.expression.clone();
    let on_expression_changed = props.on_expression_changed.clone();
    use_effect_with(props.mode, move |&mode| {
        if !mode.is_open() {
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

    let on_expression_changed = props.on_expression_changed.clone();
    let history_onclick = Callback::from(move |e: MouseEvent| {
        if let Some(on_expression_changed) = &on_expression_changed {
            if let Some(target_element) = e.target().and_then(|x| x.dyn_into::<Element>().ok()) {
                let text_content = target_element.text_content().unwrap_or_default();
                console_log!("clicked history item '{text_content}'");
                on_expression_changed.emit(text_content);
            }
        }
    });

    let on_expression_changed = props.on_expression_changed.clone();
    let palette_onclick = Callback::from(move |e: MouseEvent| {
        if let Some(on_expression_changed) = &on_expression_changed {
            if let Some(target_element) = e.target().and_then(|x| x.dyn_into::<Element>().ok()) {
                let text_content = target_element.text_content().unwrap_or_default();
                if let Some(expression) = target_element.get_attribute("data-expr") {
                    console_log!("clicked command palette item '{text_content}'");
                    on_expression_changed.emit(expression);
                }
            }
        }
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

    html! {
        <div>
        if let MenuMode::Editor = props.mode {
            <textarea class="text-white text-2xl bg-gray-800 font-normal p-8 h-screen w-screen font-mono"
                rows="5" cols="33" wrap="off" value={(*text).clone()} {oninput}/>
        } else if let MenuMode::Commands = props.mode {
            <div class="text-white text-l font-normal p-2">
                {
                    for props.expression_palette.as_array()
                        .unwrap_or(&vec![])
                        .into_iter()
                        .filter_map(|v| {
                            let value = v.get("value");
                            v.get("label")
                                .or(value)
                                .and_then(|x| x.as_str())
                                .map(|x| (x, v.as_str().unwrap_or_default().to_string()))
                        })
                        .map(|(v, data)| html! {
                            <div class="p-4 mt-2 h-12 bg-gray-800 text-ellipsis whitespace-nowrap overflow-hidden" onclick={palette_onclick.clone()} data-expr={data}>
                                { v }
                            </div>
                        })
                }
            </div>
        } else if let MenuMode::History = props.mode {
            <div class="text-white text-l font-normal p-2">
                {
                    for props.expression_history.as_array()
                        .unwrap_or(&vec![])
                        .into_iter()
                        .map(|v| html! {
                            <div class="p-4 mt-2 h-12 bg-gray-800 text-ellipsis whitespace-nowrap overflow-hidden" onclick={history_onclick.clone()}>
                                { v.as_str().unwrap_or("<unknown format>") }
                            </div>
                        })
                }
            </div>
        }
        </div>
    }
}

#[derive(Properties, PartialEq, Clone)]
struct HamburgerButtonProps {
    #[prop_or_default]
    pub focussed: bool,
    pub onclick: Callback<MouseEvent>,
}

#[function_component(HamburgerButton)]
fn menu_btn(props: &HamburgerButtonProps) -> Html {
    let onclick_clone = props.onclick.clone();
    let onclick = Callback::from(move |e: MouseEvent| {
        if let Some(active_element) = web_sys::window()
            .and_then(|x| x.document())
            .and_then(|x| x.active_element())
            .and_then(|x| x.dyn_into::<HtmlElement>().ok())
        {
            _ = active_element.blur();
        }
        onclick_clone.emit(e);
    });
    html!(
        <button class="relative group" {onclick}>
            if props.focussed {
                <div class="relative flex overflow-hidden items-center justify-center rounded-full w-[50px] h-[50px] transform transition-all bg-slate-700 ring-gray-300 hover:ring-8 ring-4 ring-opacity-30 duration-200 shadow-md">
                    <div class="flex flex-col justify-between w-[20px] h-[20px] transform transition-all duration-300 origin-center overflow-hidden">
                        <div class="bg-white h-[2px] w-7 transform transition-all duration-300 origin-left translate-y-6 delay-100"></div>
                        <div class="bg-white h-[2px] w-7 rounded transform transition-all duration-300 translate-y-6 delay-75"></div>
                        <div class="bg-white h-[2px] w-7 transform transition-all duration-300 origin-left translate-y-6"></div>

                        <div class="absolute items-center justify-between transform transition-all duration-500 top-2.5 translate-x-0 flex w-12">
                        <div class="absolute bg-white h-[2px] w-5 transform transition-all duration-500 delay-300 rotate-45"></div>
                        <div class="absolute bg-white h-[2px] w-5 transform transition-all duration-500 delay-300 -rotate-45"></div>
                        </div>
                    </div>
                </div>
            } else {
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
            }
        </button>
    )
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum MenuMode {
    Hidden,
    None,
    Editor,
    History,
    Commands,
}

impl MenuMode {
    fn toggled(self) -> Self {
        match self {
            Self::Hidden => Self::None,
            _ => Self::Hidden,
        }
    }
    fn is_open(self) -> bool {
        match self {
            Self::Hidden => false,
            _ => true,
        }
    }
}
