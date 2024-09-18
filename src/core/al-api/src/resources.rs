use std::collections::HashMap;

use serde::Deserialize;
#[derive(Debug, Deserialize)]
pub struct Resources(HashMap<String, String>);

impl Resources {
    pub fn get_filename(&self, name: &str) -> Option<&String> {
        self.0.get(name)
    }
}
