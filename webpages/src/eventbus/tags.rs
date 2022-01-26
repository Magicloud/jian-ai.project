use std::collections::HashSet;
use yew_agent::*;

pub enum Msg {
    Tags(Vec<String>),
    Reload,
}

pub struct EventBus {
    link: AgentLink<EventBus>,
    consumers: HashSet<HandlerId>,
    tags: Vec<String>,
}

impl Agent for EventBus {
    type Reach = Context<Self>;
    type Message = ();
    type Input = Msg;
    type Output = Msg;

    fn create(link: AgentLink<Self>) -> Self {
        Self {
            link,
            consumers: HashSet::new(),
            tags: vec![],
        }
    }

    fn update(&mut self, _msg: Self::Message) {}

    fn handle_input(&mut self, msg: Self::Input, _id: HandlerId) {
        match msg {
            Msg::Tags(tags) => {
                self.tags = tags;
                let _ = self
                    .consumers
                    .iter()
                    .map(|x| self.link.respond(*x, Msg::Tags(self.tags.clone())))
                    .collect::<Vec<()>>();
            }
            Msg::Reload => {
                let _ = self
                    .consumers
                    .iter()
                    .map(|x| self.link.respond(*x, Msg::Reload))
                    .collect::<Vec<()>>();
            }
        }
    }

    fn connected(&mut self, id: HandlerId) {
        self.consumers.insert(id);
        self.link.respond(id, Msg::Tags(self.tags.clone()))
    }

    fn disconnected(&mut self, id: HandlerId) {
        self.consumers.remove(&id);
    }
}
