use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

use crate::model::{identifier, time};

use super::{SearchModel, TreeModel};

#[derive(Clone, Deserialize, Serialize)]
pub struct SessionData {
    pub created_at: u64,
    pub identifier: String,
    pub last_modified: u64,
    pub name: String,
    pub search_state: SearchModel,
    pub tree_state: TreeModel,
}

impl SessionData {
    pub fn mark_modified(&mut self) {
        self.last_modified = time::seconds_now();
    }

    pub fn new(name: String) -> Self {
        assert_ne!(name, "");

        let now = time::seconds_now();

        Self {
            created_at: now,
            identifier: identifier::generate(),
            last_modified: now,
            name,
            search_state: SearchModel::default(),
            tree_state: TreeModel::default(),
        }
    }
}

#[derive(Default)]
pub struct SessionsModel {
    active_identifier: Option<String>,
    order: Vec<String>,
    session_closed_last: Option<SessionData>,
    sessions: FxHashMap<String, SessionData>,
}

impl SessionsModel {
    fn neighbor_of(&self, identifier: &str) -> Option<String> {
        let position = self.order.iter().position(|entry| entry == identifier)?;
        let neighbor = if position > 0 { position - 1 } else { 1 };

        debug_assert_ne!(neighbor, position);

        self.order.get(neighbor).cloned()
    }

    pub fn active_identifier(&self) -> Option<&str> {
        self.active_identifier.as_deref()
    }

    pub fn active_session(&self) -> Option<&SessionData> {
        self.sessions.get(self.active_identifier.as_deref()?)
    }

    pub fn active_session_mut(&mut self) -> Option<&mut SessionData> {
        self.sessions.get_mut(self.active_identifier.as_deref()?)
    }

    pub fn create_session(&mut self, name: String) -> String {
        let session = SessionData::new(name);
        let identifier = session.identifier.clone();

        self.add_session(session);
        self.active_identifier = Some(identifier.clone());

        debug_assert!(self.active_session().is_some());

        identifier
    }

    pub fn delete_session(&mut self, identifier: &str) -> Option<String> {
        if self.active_identifier.as_deref() == Some(identifier) {
            self.active_identifier = self.neighbor_of(identifier);
        }

        if let Some(session) = self.sessions.remove(identifier) {
            if !session.tree_state.nodes.is_empty() {
                self.session_closed_last = Some(session);
            }
        }

        self.order.retain(|entry| entry != identifier);

        debug_assert_eq!(self.order.len(), self.sessions.len());
        debug_assert_ne!(self.active_identifier.as_deref(), Some(identifier));

        self.active_identifier.clone()
    }

    pub fn get(&self, identifier: &str) -> Option<&SessionData> {
        self.sessions.get(identifier)
    }

    pub fn get_mut(&mut self, identifier: &str) -> Option<&mut SessionData> {
        self.sessions.get_mut(identifier)
    }

    pub fn has_restorable_session(&self) -> bool {
        if self.session_closed_last.is_some() {
            return true;
        }

        self.sessions
            .values()
            .any(|session| !session.tree_state.nodes.is_empty())
    }

    pub fn add_session(&mut self, session: SessionData) {
        assert!(identifier::is_valid(&session.identifier));

        let identifier = session.identifier.clone();

        if self.sessions.insert(identifier.clone(), session).is_none() {
            self.order.push(identifier);
        }

        debug_assert_eq!(self.order.len(), self.sessions.len());
    }

    pub fn is_empty(&self) -> bool {
        self.sessions.is_empty()
    }

    pub fn most_recent_with_tree(&self) -> Option<String> {
        self.sessions
            .values()
            .filter(|session| !session.tree_state.nodes.is_empty())
            .max_by(|left, right| {
                left.last_modified
                    .cmp(&right.last_modified)
                    .then_with(|| right.identifier.cmp(&left.identifier))
            })
            .map(|session| session.identifier.clone())
    }

    pub fn order(&self) -> &[String] {
        &self.order
    }

    pub fn ordered(&self) -> impl Iterator<Item = &SessionData> {
        self.order
            .iter()
            .filter_map(|identifier| self.sessions.get(identifier))
    }

    pub fn rebuild_order(&mut self) {
        self.order.clear();
        self.order.extend(self.sessions.keys().cloned());

        let sessions = &self.sessions;

        self.order.sort_by(|left, right| {
            let created_left = sessions.get(left).map_or(0, |session| session.created_at);
            let created_right = sessions.get(right).map_or(0, |session| session.created_at);

            created_left
                .cmp(&created_right)
                .then_with(|| left.cmp(right))
        });

        debug_assert_eq!(self.order.len(), self.sessions.len());
    }

    pub fn rename_session(&mut self, identifier: &str, name: String) {
        assert_ne!(name, "");

        if let Some(session) = self.sessions.get_mut(identifier) {
            session.name = name;
            session.mark_modified();
        }
    }

    pub fn select_session(&mut self, identifier: &str) -> Option<&SessionData> {
        if !self.sessions.contains_key(identifier) {
            return None;
        }

        self.active_identifier = Some(identifier.to_owned());

        self.sessions.get(identifier)
    }

    pub fn set_active(&mut self, identifier: Option<String>) {
        if let Some(active) = identifier.as_deref() {
            debug_assert!(self.sessions.contains_key(active));
        }

        self.active_identifier = identifier;
    }

    pub fn set_last_closed(&mut self, session: SessionData) {
        self.session_closed_last = Some(session);
    }

    pub fn sync_from_tree_and_search(&mut self, tree: &TreeModel, search: &SearchModel) {
        if let Some(session) = self.active_session_mut() {
            session.tree_state.clone_from(tree);
            session.search_state.clone_from(search);
            session.mark_modified();
        }
    }

    pub fn take_last_closed(&mut self) -> Option<SessionData> {
        self.session_closed_last.take()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn model_with(count: u64) -> SessionsModel {
        let mut model = SessionsModel::default();

        for index in 0..count {
            let mut session = SessionData::new(format!("session {index}"));

            session.created_at = index;
            model.add_session(session);
        }

        model
    }

    #[test]
    fn insert_keeps_creation_order() {
        let model = model_with(3);

        assert_eq!(model.order().len(), 3);
        assert_eq!(model.ordered().count(), 3);
    }

    #[test]
    fn rebuild_order_sorts_by_creation() {
        let mut model = SessionsModel::default();
        let mut late = SessionData::new("late".to_owned());
        let mut early = SessionData::new("early".to_owned());

        late.created_at = 20;
        early.created_at = 10;

        let late_identifier = late.identifier.clone();
        let early_identifier = early.identifier.clone();

        model.add_session(late);
        model.add_session(early);
        model.rebuild_order();

        assert_eq!(model.order()[0], early_identifier);
        assert_eq!(model.order()[1], late_identifier);
    }

    #[test]
    fn deleting_selects_the_session_to_the_left() {
        let mut model = model_with(3);
        let second = model.order()[1].clone();

        model.set_active(Some(second.clone()));

        let next = model.delete_session(&second);

        assert_eq!(next, Some(model.order()[0].clone()));
        assert_eq!(model.order().len(), 2);
    }

    #[test]
    fn deleting_the_first_session_selects_the_new_first() {
        let mut model = model_with(3);
        let first = model.order()[0].clone();

        model.set_active(Some(first.clone()));

        let next = model.delete_session(&first);

        assert_eq!(next, Some(model.order()[0].clone()));
    }

    #[test]
    fn deleting_the_last_session_clears_the_selection() {
        let mut model = model_with(1);
        let only = model.order()[0].clone();

        model.set_active(Some(only.clone()));

        assert_eq!(model.delete_session(&only), None);
        assert_eq!(model.order().len(), 0);
    }
}
