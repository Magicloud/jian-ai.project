use anyhow::{anyhow, Error};
use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};
use reqwasm::http::*;
use web_sys::HtmlInputElement;
use yew::events::Event;
use yew::prelude::*;
use yew::TargetCast;

/// https://url.spec.whatwg.org/#fragment-percent-encode-set
const FRAGMENT: &AsciiSet = &CONTROLS.add(b' ').add(b'"').add(b'<').add(b'>').add(b'`');

struct State {
    tags: Vec<String>,
    new_tag: String,
    getting_tags: bool,
    get_tags_error: Option<Error>,
    saving_tag: bool,
    save_tag_error: Option<Error>,
    show_save_msg: bool,
}

pub struct NewTagPage {
    state: State,
}

pub enum Msg {
    SaveTag,
    SaveTagOk,
    SaveTagErr(Error),
    GetTags,
    GetTagsOk(Vec<String>),
    GetTagsErr(Error),
    UINewTagValueState(String),
}

impl Component for NewTagPage {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        ctx.link().send_message(Msg::GetTags);
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
        }
    }
    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::GetTags => {
                self.state.getting_tags = true;
                ctx.link().send_future(async {
                    match Request::get("/apis/names")
                        .send()
                        .await
                        .and_then(|x| futures::executor::block_on(x.json()))
                    {
                        Ok(ts) => Msg::GetTagsOk(ts),
                        Err(e) => Msg::GetTagsErr(anyhow!("{}", e)),
                    }
                });
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
                let tag_to_save = self.state.new_tag.clone();
                ctx.link().send_future(async move {
                    match Request::get(&format!(
                        "/apis/new_names?names={}",
                        utf8_percent_encode(&tag_to_save, FRAGMENT)
                    ))
                    .send()
                    .await
                    .map_err(|e| anyhow!("{}", e))
                    .and_then(|x| {
                        if x.ok() {
                            Ok(())
                        } else {
                            Err(anyhow!("Saving tag response is not OK: {}", x.status()))
                        }
                    }) {
                        Ok(_) => Msg::SaveTagOk,
                        Err(e) => Msg::SaveTagErr(e),
                    }
                });
                true
            }
            Msg::SaveTagOk => {
                self.state.saving_tag = false;
                self.state.new_tag = "".to_string();
                ctx.link().send_message(Msg::GetTags);
                true
            }
            Msg::SaveTagErr(e) => {
                self.state.saving_tag = false;
                self.state.save_tag_error = Some(e);
                true
            }
            Msg::UINewTagValueState(v) => {
                self.state.new_tag = v;
                self.state.show_save_msg = false;
                true
            }
        }
    }
    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {<>
        <header> <h1>{"兼爱"}</h1> </header>
        <nav class="hnav"><ul>
            <li><a href="/tagging">{"标注"}</a></li>
            {"&nbsp;"}
            <li><a href="/new_tag">{"新名称"}</a></li>
        </ul></nav>
        <section>
            <article>
                <div>
                    {if self.state.show_save_msg {
                        if self.state.saving_tag {
                            html!{<div class="mask"><h1>{"正在保存……"}</h1></div>}
                        } else if let Some(e) = &self.state.save_tag_error {
                            html!{<>
                                <p>{"保存失败。"}</p>
                                <p>{e}</p>
                            </>}
                        } else {
                            html!{<p>{"保存成功。"}</p>}
                        }
                    } else {
                        html!{}
                    }}
                    <label for="tag">{"名称：（多个输入请用“,”分割）"}</label>
                    <input id="tag" type="text" value={self.state.new_tag.clone()} onchange={ctx.link().callback(move |event: Event| {
                        Msg::UINewTagValueState(event.target_dyn_into::<HtmlInputElement>().unwrap().value())
                    })} />
                    <button type="button" onclick={ctx.link().callback(move |_| Msg::SaveTag)}>{"Save"}</button>
                </div>
                <hr />
                {if self.state.getting_tags {
                    html!{<p>{"正在下载所有名称……"}</p>}
                } else if let Some(e) = &self.state.get_tags_error {
                    html!{<>
                        <p>{"下载名称失败，请刷新页面。"}</p>
                        <p>{e}</p>
                    </>}
                } else {
                    html!{<div class="grid">{self.state.tags.iter().map(|tag| html!{<div>{tag}</div>}).collect::<Html>()}</div>}
                }}
            </article>
        </section>
        <footer>{"Magicloud"}</footer>
            </>}
    }
}
