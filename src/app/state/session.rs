use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

use super::{SearchModel, TreeModel};

#[derive(Clone)]
pub struct SessionsModel {
    pub active_identifier: Option<String>,
    pub last_closed_session: Option<SessionData>,
    pub sessions: FxHashMap<String, SessionData>,
}

impl SessionsModel {
    pub fn new() -> Self {
        Self {
            active_identifier: None,
            last_closed_session: None,
            sessions: FxHashMap::default(),
        }
    }

    pub fn create_session(&mut self, name: String) -> String {
        let session = SessionData::new(name);
        let identifier = session.identifier.clone();
        self.sessions.insert(identifier.clone(), session);
        self.active_identifier = Some(identifier.clone());
        identifier
    }

    pub fn select_session(&mut self, identifier: String) -> Option<&SessionData> {
        if self.sessions.contains_key(&identifier) {
            self.active_identifier = Some(identifier.clone());
            self.sessions.get(&identifier)
        } else {
            None
        }
    }

    pub fn delete_session(&mut self, identifier: &str) -> Option<String> {
        let position = self.session_position(identifier);

        if let Some(session) = self.sessions.remove(identifier) {
            if !session.tree_state.nodes.is_empty() {
                self.last_closed_session = Some(session);
            }
        }

        if self.active_identifier.as_ref() == Some(&identifier.to_string()) {
            self.active_identifier = self.closest_left_session(position);
        }

        self.active_identifier.clone()
    }

    pub fn active_session(&self) -> Option<&SessionData> {
        self.active_identifier.as_ref().and_then(|identifier| self.sessions.get(identifier))
    }

    pub fn active_session_mut(&mut self) -> Option<&mut SessionData> {
        self.active_identifier.as_ref().and_then(|identifier| self.sessions.get_mut(identifier))
    }

    pub fn sync_from_tree_and_search(&mut self, tree: &TreeModel, search: &SearchModel) {
        if let Some(session) = self.active_session_mut() {
            session.tree_state.clone_from(tree);
            session.search_state.clone_from(search);
            session.mark_modified();
        }
    }

    pub fn session_list(&self) -> Vec<&SessionData> {
        let mut sessions: Vec<&SessionData> = self.sessions.values().collect();
        sessions.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        sessions
    }

    pub fn rename_session(&mut self, identifier: &str, new_name: String) {
        if let Some(session) = self.sessions.get_mut(identifier) {
            session.name = new_name;
            session.mark_modified();
        }
    }

    pub fn has_restorable_session(&self) -> bool {
        self.last_closed_session.is_some()
            || self.sessions.values().any(|s| !s.tree_state.nodes.is_empty())
    }

    fn session_position(&self, identifier: &str) -> usize {
        let mut session_identifiers: Vec<_> = self.sessions.keys().cloned().collect();

        session_identifiers.sort_by(|a, b| {
            let session_a = self.sessions.get(a).unwrap();
            let session_b = self.sessions.get(b).unwrap();
            session_a.created_at.cmp(&session_b.created_at)
        });

        session_identifiers.iter().position(|session_identifier| session_identifier == identifier).unwrap_or(0)
    }

    fn closest_left_session(&self, position: usize) -> Option<String> {
        if self.sessions.is_empty() {
            return None;
        }

        let mut session_identifiers: Vec<_> = self.sessions.keys().cloned().collect();

        session_identifiers.sort_by(|a, b| {
            let session_a = self.sessions.get(a).unwrap();
            let session_b = self.sessions.get(b).unwrap();
            session_a.created_at.cmp(&session_b.created_at)
        });

        if position > 0 && position - 1 < session_identifiers.len() {
            Some(session_identifiers[position - 1].clone())
        } else if !session_identifiers.is_empty() {
            Some(session_identifiers[0].clone())
        } else {
            None
        }
    }
}

impl Default for SessionsModel {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct SessionData {
    pub identifier: String,
    pub name: String,
    pub created_at: u64,
    pub last_modified: u64,
    pub tree_state: TreeModel,
    pub search_state: SearchModel,
}

impl SessionData {
    pub fn new(name: String) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            identifier: uuid::Uuid::new_v4().to_string(),
            name,
            created_at: now,
            last_modified: now,
            tree_state: TreeModel::default(),
            search_state: SearchModel::default(),
        }
    }

    pub fn mark_modified(&mut self) {
        self.last_modified = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
    }
}
