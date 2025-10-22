use crate::app::handler;
use crate::app::message::{Cmd, Msg};
use crate::app::state::{Model, UiState};

pub struct Dispatcher;

impl Dispatcher {
    pub fn dispatch(model: &mut Model, ui: &mut UiState, msg: Msg) -> Cmd {
        match msg {
            Msg::Session(msg) => handler::session::handle(model, ui, msg),
            Msg::Tree(msg) => handler::tree::handle(model, ui, msg),
            Msg::Search(msg) => handler::search::handle(model, msg),
            Msg::Copy(msg) => handler::copy::handle(model, ui, msg),
            Msg::TreeGen(msg) => handler::generator::handle(model, ui, msg),
            Msg::Options(msg) => handler::options::handle(model, ui, msg),
            Msg::Filter(msg) => handler::filter::handle(model, ui, msg),
            Msg::App(msg) => handler::app::handle(model, ui, msg),
        }
    }
}
