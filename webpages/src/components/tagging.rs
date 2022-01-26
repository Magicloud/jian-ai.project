use crate::components::*;
use anyhow::Error;
use yew::prelude::*;

struct State {
    getting_tags: bool,
    get_tags_error: Option<Error>,
    tags: Vec<String>,
}

pub struct TaggingPage {
    state: State,
}

pub enum Msg {
    ToTag(String),
    GetTags,
    GetTagsOk(Vec<String>),
    GetTagsErr(Error),
}

impl Component for TaggingPage {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        ctx.link().send_message(Msg::GetTags);
        Self {
            state: State {
                getting_tags: false,
                get_tags_error: None,
                tags: vec![],
            },
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::GetTags => {
                self.state.getting_tags = true;
                let req = Request::get("/apis/names").body(Nothing).unwrap();
                let on_done = self.link.callback(
                    move |response: Response<Json<Result<Vec<String>, Error>>>| {
                        let Json(data) = response.into_body();
                        match data {
                            Ok(tags) => Msg::GetTagsOk(tags.to_vec()),
                            Err(e) => Msg::GetTagsErr(e),
                        }
                    },
                );
                let task = FetchService::fetch(req, on_done).unwrap();
                self._task = Some(task);
                true
            }
            Msg::GetTagsOk(tags) => {
                self.state.getting_tags = false;
                self.state.tags = tags;
                true
            }
            Msg::GetTagsErr(e) => {
                self.state.getting_tags = false;
                self.state.get_tags_error = Some(e);
                true
            }
            Msg::ToTag(_filename) => true, // TODO ask tagging component to tag this one
        }
    }
    fn view(&self) -> Html {
        html! {<>
        <header> <h1>{"兼爱"}</h1> </header>
        <nav class="hnav"><ul>
            <li><a href="/tagging">{"标注"}</a></li>
            <li><a href="/new_tag">{"新名称"}</a></li>
        </ul></nav>
        <section>
            <nav>
                // <PhotoList onclick=self.link.callback(move |x| Msg::ToTag(x)) />
            </nav>
            <article>
                {if self.state.getting_tags {
                    html!{<h1>{"获取标签"}</h1>}
                } else {
                    html!{}
                }}
                // <Tagging />
            </article>
        </section>
        <footer>{"Magicloud"}</footer>
            </>}
    }
}
