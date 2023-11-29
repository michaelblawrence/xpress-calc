#![recursion_limit = "1024"]

use app::App;

mod app;

fn main() {
    yew::Renderer::<App>::new().render();
}
