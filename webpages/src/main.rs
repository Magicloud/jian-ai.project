mod components;
mod eventbus;

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::start_app::<components::base_page::BasePage>();
}
