use crate::components::new_tag::*;
use anyhow::{anyhow, Result};
use reqwasm::http::*;
use yew::prelude::*;
use yew_router::prelude::*;

enum RemoteValue<T> {
    NotStartedYet,
    Doing,
    Done(Result<T>),
}

type Tags = RemoteValue<Vec<String>>;

pub struct BasePage {
    tags: Tags,
}

pub enum Msg {
    GetTags,
    GetTagsResult(Result<Vec<String>>),
}

impl Component for BasePage {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        ctx.link().send_message(Msg::GetTags);
        Self {
            tags: RemoteValue::NotStartedYet,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::GetTags => {
                self.tags = RemoteValue::Doing;
                ctx.link().send_future(async {
                    match Request::get("http://localhost:8000/apis/names")
                        .send()
                        .await
                    {
                        Ok(resp) => match resp.json().await {
                            Ok(ts) => Msg::GetTagsResult(Ok(ts)),
                            Err(e) => Msg::GetTagsResult(Err(anyhow!("{}", e))),
                        },
                        Err(e) => Msg::GetTagsResult(Err(anyhow!("{}", e))),
                    }
                })
            }
            Msg::GetTagsResult(x) => {
                self.tags = RemoteValue::Done(x);
            }
        };
        true
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {<BrowserRouter>
            <header class="d-flex flex-wrap justify-content-center py-3 mb-4 border-bottom">
                <a href="/" class="d-flex align-items-center mb-3 mb-md-0 me-md-auto text-dark text-decoration-none"><h1>{"兼爱"}</h1></a>
                <nav><Navigator /></nav>
            </header>
            <section>
                <article>
                    <div>
                        {match &self.tags {
                            RemoteValue::Doing => {
                                html! {<h1>{"正在获取所有名称……"}</h1>}
                            }
                            RemoteValue::Done(Err(e)) => {
                                html! {<h1>{format!("获取所有名称失败 {}", e)}</h1>}
                            }
                            _ => html! {}
                        }}
                    </div>
                    <ContextProvider<Vec<String>> context={
                        if let RemoteValue::Done(Ok(tags)) = &(self.tags) {
                            tags.clone()
                        } else {
                            vec![]
                        }
                    }>
                        <Switch<Route> render={Switch::render(switch)} />
                    </ContextProvider<Vec<String>>>
                </article>
            </section>
            <footer>{"Magicloud"}</footer>
        </BrowserRouter>}
    }
}

#[derive(Routable, PartialEq, Clone, Debug)]
pub enum Route {
    #[not_found]
    #[at("/404")]
    NotFound,
    #[at("/tagging")]
    Tagging,
    #[at("/new_tag")]
    NewTag,
}

fn switch(routes: &Route) -> Html {
    match routes {
        Route::NotFound => html! { <h1>{"404"}</h1> },
        Route::Tagging => html! { <h1>{ "Tagging" }</h1> },
        Route::NewTag => html! { <NewTag /> },
    }
}

#[function_component(Navigator)]
fn navigator() -> Html {
    let curr_r = use_location().and_then(|x| x.route::<Route>());
    html! { <ul class="nav nav-pills">
        <li class="nav-item"><Link<Route> classes={classes!({
            if curr_r == Some(Route::Tagging) {
                "nav-link active"
            } else {
                "nav-link"
            }
        })} to={Route::Tagging}>{ "标注" }</Link<Route>></li>
        <li class="nav-item"><Link<Route> classes={classes!({
            if curr_r == Some(Route::NewTag) {
                "nav-link active"
            } else {
                "nav-link"
            }
        })} to={Route::NewTag}>{ "新名称" }</Link<Route>></li>
    </ul> }
}
