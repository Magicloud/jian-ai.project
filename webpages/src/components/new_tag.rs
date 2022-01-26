use crate::eventbus;
use anyhow::{anyhow, Error, Result};
use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};
use reqwasm::http::*;
use web_sys::HtmlInputElement;
use yew::events::Event;
use yew::prelude::*;
use yew::TargetCast;
use yew_agent::{Bridge, Bridged, Dispatched, Dispatcher};

/// https://url.spec.whatwg.org/#fragment-percent-encode-set
const FRAGMENT: &AsciiSet = &CONTROLS.add(b' ').add(b'"').add(b'<').add(b'>').add(b'`');

enum RemoteWrite {
    NotStartedYet,
    Doing,
    Done(Option<Error>),
}

pub struct NewTag {
    new_tag: String,
    tags: Vec<String>,
    persist_tags: RemoteWrite,
    _eb_tags: Box<dyn Bridge<eventbus::tags::EventBus>>,
    eb_tags: Dispatcher<eventbus::tags::EventBus>,
}

pub enum Msg {
    SaveTag,
    SaveTagsResult(Result<()>),
    UINewTagValueState(String),
    EB_MSG(eventbus::tags::Msg),
}

impl Component for NewTag {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            new_tag: "".to_string(),
            tags: vec![],
            persist_tags: RemoteWrite::NotStartedYet,
            _eb_tags: eventbus::tags::EventBus::bridge(ctx.link().callback(Msg::EB_MSG)),
            eb_tags: eventbus::tags::EventBus::dispatcher(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::SaveTag => {
                self.persist_tags = RemoteWrite::Doing;
                let tag_to_save = self.new_tag.clone();
                ctx.link().send_future(async move {
                    Msg::SaveTagsResult(
                        Request::post(&format!(
                            "http://localhost:8000/apis/new_names?names={}",
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
                        }),
                    )
                });
            }
            Msg::SaveTagsResult(r) => {
                if let Ok(()) = r {
                    self.eb_tags.send(eventbus::tags::Msg::Reload);
                };
                self.persist_tags = RemoteWrite::Done(r.err());
            }
            Msg::UINewTagValueState(v) => {
                self.new_tag = v;
            }
            Msg::EB_MSG(msg) => {
                if let eventbus::tags::Msg::Tags(tags) = msg {
                    self.tags = tags;
                }
            }
        };
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {<>
            <div>
                {match &self.persist_tags {
                    RemoteWrite::Doing => {html!{<div class="mask"><h1>{"正在保存……"}</h1></div>}}
                    RemoteWrite::Done(None) => {html!{<p>{"保存成功。"}</p>}}
                    RemoteWrite::Done(Some(e)) => {html!{<>
                            <p>{"保存失败。"}</p>
                            <p>{e}</p>
                        </>}}
                    RemoteWrite::NotStartedYet => {html!{}}
                }}
                <label for="tag">{"名称：（多个输入请用“,”分割）"}</label>
                <input id="tag" type="text" value={self.new_tag.clone()} onchange={ctx.link().callback(move |event: Event| {
                    Msg::UINewTagValueState(event.target_dyn_into::<HtmlInputElement>().unwrap().value())
                })} />
                <button type="button" onclick={ctx.link().callback(move |_| Msg::SaveTag)}>{"Save"}</button>
            </div>
            <hr />
            <div class="grid">{self.tags.iter().map(|tag| html!{<div>{tag}</div>}).collect::<Html>()}</div>
        </>}
    }
}
