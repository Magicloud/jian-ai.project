use crate::components::Photo;
use crate::event_buses::*;
use anyhow::Error;
use yew::format::*;
use yew::prelude::*;
use yew::services::fetch::*;

pub struct PhotoList {
    state: State,
    props: Props,
    link: ComponentLink<Self>,
    event_bus_item: Box<dyn Bridge<ItemToListEventBus>>,
    task: Option<FetchTask>,
}

struct State {
    photos: Vec<String>,
    getting_photo_list: bool,
    get_photo_list_error: Option<Error>,
}

pub enum Msg {
    GetPhotoList,
    GetPhotoListOk(Vec<String>),
    GetPhotoListErr(Error),
    Next(()),
}

#[derive(Clone, Properties)]
pub struct Props {
    pub onclick: Callback<String>,
}

impl Component for PhotoList {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        link.send_message(Msg::GetPhotoList);
        Self {
            state: State {
                photos: vec![],
                getting_photo_list: false,
                get_photo_list_error: None,
            },
            props,
            event_bus_item: ItemToListEventBus::bridge(link.callback(Msg::Next)),
            link,
            task: None,
        }
    }
    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::GetPhotoList => {
                self.state.getting_photo_list = true;
                let req = Request::get("/apis/unnamed_images").body(Nothing).unwrap();
                let on_done = self.link.callback(
                    move |response: Response<Json<Result<Vec<String>, Error>>>| {
                        let Json(data) = response.into_body();
                        match data {
                            Ok(photo_list) => Msg::GetPhotoListOk(photo_list.to_vec()),
                            Err(e) => Msg::GetPhotoListErr(e),
                        }
                    },
                );
                let task = FetchService::fetch(req, on_done).unwrap();
                self.task = Some(task);
                true
            }
            Msg::GetPhotoListErr(e) => {
                self.state.getting_photo_list = false;
                self.state.get_photo_list_error = Some(e);
                true
            }
            Msg::GetPhotoListOk(strs) => {
                self.state.getting_photo_list = false;
                self.state.photos = strs;
                true
            }
            Msg::Next(_) => {
                yew::utils::document()
                    .get_element_by_id("nav")
                    .unwrap()
                    .scroll_by_with_x_and_y(0.0, 300.0);
                true
            }
        }
    }
    fn change(&mut self, _: Self::Properties) -> ShouldRender {
        true
    }
    fn view(&self) -> Html {
        if self.state.getting_photo_list {
            html! {<div>{"Loading..."}</div>}
        } else {
            if let Some(e) = &self.state.get_photo_list_error {
                html! {<div>{format!("{}", e)}</div>}
            } else {
                html! {
                    <div class="images"><ul class="images">{self.state.photos.iter().map(|filename| html! {<li><Photo photo={filename.clone()}/></li>}).collect::<Html>()}</ul></div>
                    // <ul>{self.state.photos.iter().map(|y| {
                    //     let x = y.clone();
                    //     let z = y.clone();
                    //     let onclick = {
                    //         self.link.callback(move |()| Msg::ToTag(x.clone()));
                    //         self.props.onclick.reform(move |_| z.clone())
                    //     };
                    //     html! {<li><img id={y.to_string()} src={format!("/pics/{}", y)} loading="lazy" onclick=onclick /></li>}
                    // }).collect::<Html>()}</ul>
                }
            }
        }
    }
}
