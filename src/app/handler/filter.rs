use crate::app::message::{Command, Filter};
use crate::app::state::{Model, UiState};
use crate::model::options::PatternList;

use super::options::commit_options;

pub fn handle(model: &mut Model, ui: &mut UiState, message: Filter) -> Command {
    match message {
        Filter::Added { list, pattern } => handle_added(model, ui, list, &pattern),
        Filter::Removed { index, list } => {
            let mut options = (*model.options).clone();

            if options.remove_pattern(list, index) {
                commit_options(model, ui, options);
            }
        }
        Filter::Reset(list) => {
            let mut options = (*model.options).clone();

            options.reset_patterns(list);
            commit_options(model, ui, options);
        }
    }

    Command::None
}

fn handle_added(model: &mut Model, ui: &mut UiState, list: PatternList, pattern: &str) {
    let mut options = (*model.options).clone();

    if let Err(error) = options.add_pattern(list, pattern) {
        ui.toast
            .error(format!("The pattern was not added: {error}"));

        return;
    }

    commit_options(model, ui, options);

    match list {
        PatternList::Exclude => ui.pattern_exclude_new.clear(),
        PatternList::Include => ui.pattern_include_new.clear(),
    }
}
