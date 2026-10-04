use crate::app::handler;
use crate::app::message::{Command, Message};
use crate::app::state::{Model, UiState};

pub fn dispatch(model: &mut Model, ui: &mut UiState, message: Message) -> Command {
    match message {
        Message::App(app) => handler::app::handle(model, ui, app),
        Message::Copy(copy) => handler::copy::handle(model, ui, copy),
        Message::Filter(filter) => handler::filter::handle(model, ui, filter),
        Message::Notice(notice) => handler::notice::handle(ui, notice),
        Message::Options(options) => handler::options::handle(model, ui, options),
        Message::Preset(preset) => handler::preset::handle(model, ui, preset),
        Message::Render(render) => handler::render::handle(model, ui, render),
        Message::Search(search) => handler::search::handle(model, ui, search),
        Message::Session(session) => handler::session::handle(model, ui, session),
        Message::Skeleton(skeleton) => handler::skeleton::handle(model, ui, skeleton),
        Message::Tree(tree) => handler::tree::handle(model, ui, tree),
    }
}
