mod pages;

use crate::pages::new_tag::*;
use yew::prelude::*;
use yew_router::prelude::*;

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::start_app::<App>();
}

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <BrowserRouter>
            <Switch<Route> render={Switch::render(switch)} />
        </BrowserRouter>
    }
}

#[derive(Routable, PartialEq, Clone)]
enum Route {
    #[at("/tagging")]
    TaggingPage,
    #[at("/new_tag")]
    NewTagPage,
}

fn switch(routes: &Route) -> Html {
    match routes {
        Route::TaggingPage => html! { <h1>{ "Tagging" }</h1> },
        Route::NewTagPage => html! { <NewTagPage />},
    }
}
