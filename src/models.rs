use std::collections::HashMap;

use serde::{Serialize, Deserialize};

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct DatabaseState {
    #[serde(rename = "lastItemId")]
    pub last_item_id: Option<u32>,
    pub epics: HashMap<u32, Epic>,
    pub stories: HashMap<u32, Story>,
}

#[derive(Clone, Debug)]
pub enum Feature {
    Epic(Epic),
    Story(Story),
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum Status {
    #[serde(rename = "open")]
    Open,
    #[serde(rename = "inProgress")]
    InProgress,
    #[serde(rename = "resolved")]
    Resolved,
    #[serde(rename = "closed")]
    Closed,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Epic {
    pub name: String,
    pub description: String,
    pub status: Status,
    #[serde(rename = "storyIds")]
    pub story_ids: Vec<u32>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Story {
    pub name: String,
    pub description: String,
    pub status: Status,
}

impl Epic {
    pub fn new(name: &str, description: &str) -> Self {
        Self {
            name: name.to_owned(),
            description: description.to_owned(),
            status: Status::Open,
            story_ids: vec![],
        }
    }
}

impl Story {
    pub fn new(name: &str, description: &str) -> Self {
        Self {
            name: name.to_owned(),
            description: description.to_owned(),
            status: Status::Open
        }
    }
}
