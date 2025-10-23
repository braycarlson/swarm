use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use super::{SearchModel, TreeModel};

#[derive(Clone)]
pub struct SessionsModel {
    pub active_id: Option<String>,
    pub sessions: HashMap<String, SessionData>,
}

impl SessionsModel {
    pub fn new() -> Self {
        Self {
            active_id: None,
            sessions: HashMap::new(),
        }
    }

    pub fn create_session(&mut self, name: String) -> String {
        let session = SessionData::new(name);
        let id = session.id.clone();
        self.sessions.insert(id.clone(), session);
        self.active_id = Some(id.clone());
        id
    }

    pub fn select_session(&mut self, id: String) -> Option<&SessionData> {
        if self.sessions.contains_key(&id) {
            self.active_id = Some(id.clone());
            self.sessions.get(&id)
        } else {
            None
        }
    }

    pub fn delete_session(&mut self, id: &str) -> Option<String> {
        let position = self.session_position(id);
        self.sessions.remove(id);

        if self.active_id.as_ref() == Some(&id.to_string()) {
            self.active_id = self.closest_left_session(position);
        }

        self.active_id.clone()
    }

    pub fn active_session(&self) -> Option<&SessionData> {
        self.active_id.as_ref().and_then(|id| self.sessions.get(id))
    }

    pub fn active_session_mut(&mut self) -> Option<&mut SessionData> {
        self.active_id.as_ref().and_then(|id| self.sessions.get_mut(id))
    }

    pub fn sync_from_tree_and_search(&mut self, tree: TreeModel, search: SearchModel) {
        if let Some(session) = self.active_session_mut() {
            session.tree_state = tree;
            session.search_state = search;
            session.mark_modified();
        }
    }

    pub fn session_list(&self) -> Vec<&SessionData> {
        let mut sessions: Vec<&SessionData> = self.sessions.values().collect();
        sessions.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        sessions
    }

    pub fn rename_session(&mut self, id: &str, new_name: String) {
        if let Some(session) = self.sessions.get_mut(id) {
            session.name = new_name;
            session.mark_modified();
        }
    }

    fn session_position(&self, id: &str) -> usize {
        let mut ids: Vec<_> = self.sessions.keys().cloned().collect();

        ids.sort_by(|a, b| {
            let sa = self.sessions.get(a).unwrap();
            let sb = self.sessions.get(b).unwrap();
            sa.created_at.cmp(&sb.created_at)
        });

        ids.iter().position(|sid| sid == id).unwrap_or(0)
    }

    fn closest_left_session(&self, position: usize) -> Option<String> {
        if self.sessions.is_empty() {
            return None;
        }

        let mut ids: Vec<_> = self.sessions.keys().cloned().collect();

        ids.sort_by(|a, b| {
            let sa = self.sessions.get(a).unwrap();
            let sb = self.sessions.get(b).unwrap();
            sa.created_at.cmp(&sb.created_at)
        });

        if position > 0 && position - 1 < ids.len() {
            Some(ids[position - 1].clone())
        } else if !ids.is_empty() {
            Some(ids[0].clone())
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
    pub id: String,
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
            id: uuid::Uuid::new_v4().to_string(),
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
