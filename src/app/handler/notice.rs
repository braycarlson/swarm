use crate::app::message::{Command, Notice};
use crate::app::state::UiState;

pub fn handle(ui: &mut UiState, message: Notice) -> Command {
    match message {
        Notice::Error(text) => ui.toast.error(text),
        Notice::Success(text) => ui.toast.success(text),
    }

    Command::None
}
