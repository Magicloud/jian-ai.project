use anyhow::Error;
use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};
use yew::format::*;
use yew::prelude::*;
use yew::services::console::ConsoleService;
use yew::services::fetch::*;

/// https://url.spec.whatwg.org/#fragment-percent-encode-set
const FRAGMENT: &AsciiSet = &CONTROLS.add(b' ').add(b'"').add(b'<').add(b'>').add(b'`');

struct State {
    tags: Vec<String>,
    new_tag: String,
    getting_tags: bool,
    get_tags_error: Option<FetchErrors>,
    saving_tag: bool,
    save_tag_error: Option<FetchErrors>,
    show_save_msg: bool,
}

pub struct NewTagPage {
    state: State,
    link: ComponentLink<Self>,
    _task: Option<FetchTask>,
}

pub enum Msg {
    SaveTag,
    SaveTagOk,
    SaveTagErr(FetchErrors),
    GetTags,
    GetTagsOk(Vec<String>),
    GetTagsErr(FetchErrors),
    UINewTagValueState(ChangeData),
}

pub enum FetchErrors {
    FetchError(Error),
    RequestError(StatusCode),
    ContentError(Error),
}
impl std::fmt::Display for FetchErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FetchErrors::FetchError(e) => write!(f, "Error starting fetch: {}", e),
            FetchErrors::RequestError(s) => write!(f, "Server returns error code: {}", s),
            FetchErrors::ContentError(e) => write!(f, "Server returns unrecognized data: {}", e),
        }
    }
}

impl Component for NewTagPage {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        link.send_message(Msg::GetTags);
        Self {
            state: State {
                tags: vec![],
                new_tag: "".to_string(),
                getting_tags: false,
                get_tags_error: None,
                saving_tag: false,
                save_tag_error: None,
                show_save_msg: false,
            },
            link,
            _task: None,
        }
    }
    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::GetTags => {
                self.state.getting_tags = true;
                let req = Request::get("/apis/names").body(Nothing).unwrap();
                let on_done = self.link.callback(
                    move |response: Response<Json<Result<Vec<String>, Error>>>| {
                        if response.status().is_success() {
                            let Json(data) = response.into_body();
                            match data {
                                Ok(tags) => Msg::GetTagsOk(tags.to_vec()),
                                Err(e) => Msg::GetTagsErr(FetchErrors::ContentError(e)),
                            }
                        } else {
                            Msg::GetTagsErr(FetchErrors::RequestError(response.status()))
                        }
                    },
                );
                match FetchService::fetch(req, on_done) {
                    Ok(task) => self._task = Some(task),
                    Err(e) => self
                        .link
                        .send_message(Msg::GetTagsErr(FetchErrors::FetchError(e))),
                }
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
            Msg::SaveTag => {
                self.state.saving_tag = true;
                self.state.show_save_msg = true;
                match Request::post(format!(
                    "/apis/new_names?names={}",
                    utf8_percent_encode(&self.state.new_tag, FRAGMENT)
                ))
                .body(Nothing)
                {
                    Ok(req) => {
                        let on_done =
                            self.link
                                .callback(move |response: Response<Result<String, Error>>| {
                                    if response.status().is_success() {
                                        Msg::SaveTagOk
                                    } else {
                                        Msg::SaveTagErr(FetchErrors::RequestError(
                                            response.status(),
                                        ))
                                    }
                                });
                        match FetchService::fetch(req, on_done) {
                            Ok(task) => self._task = Some(task),
                            Err(e) => self
                                .link
                                .send_message(Msg::GetTagsErr(FetchErrors::FetchError(e))),
                        }
                        true
                    }
                    Err(e) => {
                        ConsoleService::error(&e.to_string());
                        true
                    }
                }
            }
            Msg::SaveTagOk => {
                self.state.saving_tag = false;
                self.state.new_tag = "".to_string();
                self.link.send_message(Msg::GetTags);
                true
            }
            Msg::SaveTagErr(e) => {
                self.state.saving_tag = false;
                self.state.save_tag_error = Some(e);
                true
            }
            Msg::UINewTagValueState(v) => {
                match v {
                    ChangeData::Value(v) => self.state.new_tag = v,
                    ChangeData::Select(_) => {}
                    ChangeData::Files(_) => {}
                }
                self.state.show_save_msg = false;
                true
            }
        }
    }
    fn change(&mut self, _: Self::Properties) -> ShouldRender {
        true
    }
    fn view(&self) -> Html {
        html! {<>
        <header> <h1>{"兼爱"}</h1> </header>
        <nav class="hnav"><ul>
            <li><a href="/tagging">{"标注"}</a></li>
            <li><a href="/new_tag">{"新名称"}</a></li>
        </ul></nav>
        <section>
            <article>
                <div>
                    {if self.state.show_save_msg {
                        if self.state.saving_tag {
                            html!{<div class="mask"><h1>{"正在保存……"}</h1></div>}
                        } else {
                            if let Some(e) = &self.state.save_tag_error {
                                html!{<>
                                    <p>{"保存失败。"}</p>
                                    <p>{e}</p>
                                </>}
                            } else {
                                html!{<p>{"保存成功。"}</p>}
                            }
                        }
                    } else {
                        html!{}
                    }}
                    <label for="tag">{"名称：（多个输入请用“,”分割）"}</label>
                    <input id="tag" type="text" value={self.state.new_tag.clone()} onchange=self.link.callback( move |v| Msg::UINewTagValueState(v)) />
                    <button type="button" onclick=self.link.callback(move |_| Msg::SaveTag)>{"Save"}</button>
                </div>
                <hr />
                {if self.state.getting_tags {
                    html!{<p>{"正在下载所有名称……"}</p>}
                } else {
                    if let Some(e) = &self.state.get_tags_error {
                        html!{<>
                            <p>{"下载名称失败，请刷新页面。"}</p>
                            <p>{e}</p>
                        </>}
                    } else {
                        html!{<div class="grid">{self.state.tags.iter().map(|tag| html!{<div>{tag}</div>}).collect::<Html>()}</div>}
                    }
                }}
            </article>
        </section>
        <footer>{"Magicloud"}</footer>
            </>}
    }
}
