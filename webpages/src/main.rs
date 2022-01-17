fn main() {
    yew::start_app::<App>();
}

use yew::prelude::*;

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <BrowserRouter>
            <Switch<Route> render={Switch::render(switch)} />
        </BrowserRouter>
    }
}

use yew_router::prelude::*;

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
        Route::NewTagPage => html! { <h1>{ "NewTag" }</h1> },
    }
}
