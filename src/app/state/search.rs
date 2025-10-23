use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Deserialize, Serialize)]
pub struct SearchModel {
    pub query: String,
    pub active: bool,
}

impl SearchModel {
    pub fn has_query(&self) -> bool {
        !self.query.is_empty() && self.active
    }

    pub fn activate(&mut self) {
        self.active = true;
    }

    pub fn clear(&mut self) {
        self.query.clear();
        self.active = false;
    }

    pub fn set_query(&mut self, query: String) {
        self.query = query;
        self.active = true;
    }
}
