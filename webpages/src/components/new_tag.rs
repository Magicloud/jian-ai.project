use anyhow::{anyhow, Error, Result};
use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};
use reqwasm::http::*;
use web_sys::HtmlInputElement;
use yew::events::Event;
use yew::prelude::*;
use yew::TargetCast;

/// https://url.spec.whatwg.org/#fragment-percent-encode-set
const FRAGMENT: &AsciiSet = &CONTROLS.add(b' ').add(b'"').add(b'<').add(b'>').add(b'`');

// need tag management

enum RemoteWrite {
    NotStartedYet,
    Doing,
    Done(Option<Error>),
}

pub struct NewTag {
    new_tag: String,
    persist_tags: RemoteWrite,
}

pub enum Msg {
    SaveTag,
    SaveTagsResult(Result<()>),
    UINewTagValueState(String),
}

impl Component for NewTag {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            new_tag: "".to_string(),
            persist_tags: RemoteWrite::NotStartedYet,
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
                self.persist_tags = RemoteWrite::Done(r.err());
            }
            Msg::UINewTagValueState(v) => {
                self.new_tag = v;
            }
        };
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let (tags, _) = ctx
            .link()
            .context::<Vec<String>>(Callback::noop())
            .expect("Context tags is not set");
        html! {<>
            <div>
                {match &self.persist_tags {
                    RemoteWrite::Doing => {html!{<div class="mask"><h1>{"??????????????????"}</h1></div>}}
                    RemoteWrite::Done(None) => {html!{<p>{"???????????????"}</p>}}
                    RemoteWrite::Done(Some(e)) => {html!{<>
                            <p>{"???????????????"}</p>
                            <p>{e}</p>
                        </>}}
                    RemoteWrite::NotStartedYet => {html!{}}
                }}
                <label for="tag">{"?????????????????????????????????,????????????"}</label>
                <input id="tag" type="text" value={self.new_tag.clone()} onchange={ctx.link().callback(move |event: Event| {
                    Msg::UINewTagValueState(event.target_dyn_into::<HtmlInputElement>().unwrap().value())
                })} />
                <button type="button" onclick={ctx.link().callback(move |_| Msg::SaveTag)}>{"Save"}</button>
            </div>
            <hr />
            <div class="grid">{
                tags.iter().map(|tag|
                    html!{<div>{tag}</div>}
                ).collect::<Html>()
            }</div>
        </>}
    }
}
