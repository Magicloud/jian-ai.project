use crate::components::base_page::RemoteValue;
use anyhow::{anyhow, Result};
use linked_hash_set::LinkedHashSet;
use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};
use reqwasm::http::*;
use std::sync::Arc;
use yew::prelude::*;

type Photos = RemoteValue<LinkedHashSet<String>>;

// Also in new_tag.
const FRAGMENT: &AsciiSet = &CONTROLS.add(b' ').add(b'"').add(b'<').add(b'>').add(b'`');
enum RemoteWrite {
    NotStartedYet,
    Doing,
    Done(Result<String>),
}

pub struct Tagging {
    photos: Arc<Photos>,
    current_photo: Option<String>,
    current_name: Option<String>,
    persist_name: RemoteWrite,
}

pub enum Msg {
    GetPhotos,
    GetPhotosResult(Result<Vec<String>>),
    PhotoClicked(String),
    NameClicked(String),
    Save,
    SaveResult(Result<String>),
    Next,
}

impl Component for Tagging {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        ctx.link().send_message(Msg::GetPhotos);
        Self {
            photos: Arc::new(RemoteValue::NotStartedYet),
            current_photo: None,
            current_name: None,
            persist_name: RemoteWrite::NotStartedYet,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::GetPhotos => {
                self.photos = Arc::new(RemoteValue::Doing);
                ctx.link().send_future(async {
                    match Request::get("http://localhost:8000/apis/unnamed_images")
                        .send()
                        .await
                    {
                        Ok(resp) => match resp.json().await {
                            Ok(ps) => Msg::GetPhotosResult(Ok(ps)),
                            Err(e) => Msg::GetPhotosResult(Err(anyhow!("{}", e))),
                        },
                        Err(e) => Msg::GetPhotosResult(Err(anyhow!("{}", e))),
                    }
                })
            }
            Msg::GetPhotosResult(x) => match x {
                Ok(v_photos) => {
                    ctx.link()
                        .send_message(Msg::PhotoClicked(v_photos[0].clone()));
                    self.photos = Arc::new(RemoteValue::Done(Ok(v_photos.into_iter().collect())));
                }
                Err(e) => {
                    self.photos = Arc::new(RemoteValue::Done(Err(e)));
                }
            },
            Msg::PhotoClicked(i) => {
                self.current_photo = Some(i);
            }
            Msg::NameClicked(n) => {
                self.current_name = Some(n);
            }
            Msg::Save => {
                self.current_photo.clone().map(|photo| {
                    self.current_name.clone().map(|name| {
                        self.persist_name = RemoteWrite::Doing;
                        ctx.link().send_future(async move {
                            Msg::SaveResult(
                                Request::post(&format!(
                                "http://localhost:8000/apis/name_image?photo_filename={}&name={}",
                                photo,
                                utf8_percent_encode(&name, FRAGMENT)
                            ))
                                .send()
                                .await
                                .map_err(|e| anyhow!("{}", e))
                                .and_then(|x| {
                                    if x.ok() {
                                        Ok(photo)
                                    } else {
                                        Err(anyhow!(
                                            "Naming photo response is not OK: {}",
                                            x.status()
                                        ))
                                    }
                                }),
                            )
                        });
                    })
                });
            }
            Msg::SaveResult(r) => {
                let _ = &r.as_ref().map(|saved_photo| {
                    Arc::get_mut(&mut self.photos).map(|photos| {
                        photos.update(|ps| {
                            ps.remove(saved_photo);
                        })
                    })
                });
                self.persist_name = RemoteWrite::Done(r);
            }
            Msg::Next => {
                let current_photo = self.current_photo.clone();
                let _ = current_photo.map(|curr_photo| {
                    let tmp: &Photos = &self.photos;
                    if let RemoteValue::Done(Ok(photos)) = tmp {
                        let mut iter = photos.iter();
                        while let Some(photo) = iter.next() {
                            if *photo == *curr_photo {
                                if let Some(next_photo) = iter.next() {
                                    self.current_photo = Some(next_photo.clone());
                                } else if let Some(first_photo) = photos.front() {
                                    self.current_photo = Some(first_photo.clone());
                                } else {
                                    self.current_photo = None;
                                }
                                break;
                            }
                        }
                    } else {
                        self.current_photo = None;
                    }
                });
            }
        };
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {<main>
            <div class="d-flex flex-column align-items-stretch flex-shrink-0 bg-white" style="width: 230px;">
                <p class="d-flex align-items-center flex-shrink-0 p-3 link-dark text-decoration-none border-bottom fs-5 fw-semibold">{"Unnamed Photos"}</p>
                <div class="list-group list-group-flush border-bottom scrollarea">
                    {match &*self.photos {
                        RemoteValue::Done(Ok(photos)) => {
                            photos.iter().enumerate().map(|(i, photo_)| {
                                let photo = photo_.clone();
                                let p = photo.clone();
                                let cls = if self.current_photo == Some(photo.clone()) {
                                    classes!("list-group-item", "list-group-item-action", "py-3", "lh-tight", "d-flex", "w-100", "align-items-center", "justify-content-between", "active")
                                } else {
                                    classes!("list-group-item", "list-group-item-action", "py-3", "lh-tight", "d-flex", "w-100", "align-items-center", "justify-content-between")
                                };
                                html! {
                                <div class={cls} aria-current={if self.current_photo == Some(photo.clone()) {"true"} else {"false"}} onclick={
                                    ctx.link().callback(move |_| {
                                        Msg::PhotoClicked(p.clone())
                                    })
                                }>
                                    <span class="mb-1">{i}</span>
                                    <img src={format!("http://localhost:8000/pics/{}", photo)} class="mb-1" alt={photo.clone()} loading="lazy" />
                                </div>
                            }}).collect()
                        }
                        RemoteValue::Done(Err(e)) => {
                            html! {<h1>{format!("获取未命名照片失败 {}", e)}</h1>}
                        }
                        RemoteValue::Doing => {
                            html! {<h1>{"获取未命名照片……"}</h1>}
                        }
                        _ => html!{}
                    }}
                </div>
            </div>
            {if let Some(curr_photo) = self.current_photo.clone() {
                let (tags, _) = ctx
                    .link()
                    .context::<Vec<String>>(Callback::noop())
                    .expect("Context tags is not set");
                html! {<div class="tag-layout">
                    <img style="grid-area: photo;" src={format!("http://localhost:8000/pics/{}", &curr_photo)} alt={curr_photo.clone()} />

                    <div class="pt-0 mx-0 rounded-3 shadow overflow-hidden" style="grid-area: names;">
                        <form class="p-2 mb-2 bg-light border-bottom">
                            <input type="search" class="form-control" autocomplete="false" placeholder="Type to filter..." />
                        </form>
                        <div style="height: 100vh; overflow-y: scroll;">
                        <ul class="list-unstyled mb-0">
                            {tags.iter().map(|tag| {
                                let tag_ = tag.clone();
                                html!{<li class="dropdown-item d-flex align-items-center gap-2 py-2" onclick={ctx.link().callback(move |_| Msg::NameClicked(tag_.clone()))}>
                                    {tag}
                                </li>}
                            }).collect::<Html>()}
                        </ul>
                        </div>
                    </div>

                    <div style="grid-area: name;">
                        <label>{if let Some(current_name) = self.current_name.clone() {current_name} else {"".to_string()}}</label>
                    </div>

                    <div style="grid-area: buttons;">
                        <button type="button" onclick={ctx.link().callback(|_| Msg::Save)}>{"Save"}</button>
                        <button type="button" onclick={ctx.link().callback(|_| Msg::Next)}>{"Next"}</button>
                    </div>
                </div>}
            } else {html!{}}}
        </main>}
    }
}
