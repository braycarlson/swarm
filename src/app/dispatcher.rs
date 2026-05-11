use crate::app::handler;
use crate::app::message::{Command, Message};
use crate::app::state::{Model, UiState};

pub struct Dispatcher;

impl Dispatcher {
    pub fn dispatch(model: &mut Model, ui: &mut UiState, message: Message) -> Command {
        match message {
            Message::Session(message) => handler::session::handle(model, ui, message),
            Message::Tree(message) => handler::tree::handle(model, ui, message),
            Message::Search(message) => handler::search::handle(model, ui, message),
            Message::Copy(message) => handler::copy::handle(model, ui, message),
            Message::Render(message) => handler::render::handle(model, ui, message),
            Message::Skeleton(message) => handler::skeleton::handle(model, ui, message),
            Message::Options(message) => handler::options::handle(model, ui, message),
            Message::Filter(message) => handler::filter::handle(model, ui, message),
            Message::Preset(message) => handler::preset::handle(model, ui, message),
            Message::App(message) => handler::app::handle(model, ui, message),
        }
    }
}
