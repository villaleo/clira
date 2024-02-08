#![allow(dead_code)]

use std::{collections::HashMap, fmt::Display};

use serde::{Deserialize, Serialize};

/// An `Action` represents the different types of actions that are accepted from
/// user input.
#[derive(Debug, PartialEq, Eq)]
pub enum Action {
    NavigateToEpicDetail { epic_id: u32 },
    NavigateToStoryDetail { story_id: u32, epic_id: u32 },
    NavigateToPreviousPage,
    CreateEpic,
    CreateStory { epic_id: u32 },
    UpdateEpicName { epic_id: u32 },
    UpdateEpicDescription { epic_id: u32 },
    UpdateEpicStatus { epic_id: u32 },
    UpdateStoryStatus { story_id: u32 },
    DeleteEpic { epic_id: u32 },
    DeleteStory { story_id: u32, epic_id: u32 },
    Exit,
}

/// `DatabaseState` represents the state of the database. It is the base type that is
/// serialized into the JSON file for persistence.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct DatabaseState {
    #[serde(rename = "lastItemId")]
    pub last_item_id: Option<u32>,
    pub epics: HashMap<u32, Epic>,
    pub stories: HashMap<u32, Story>,
}

/// `Status` models the different states that an `Epic` or `Story` can be in. `Open` is
/// the default state.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
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

/// `Epic` represents an epic in the `JiraDatabase`. It is a high-level milestone that can
/// be broken down into smaller, achievable chunks. These chunks are called stories. Epics
/// may have many children stories.
///
/// If an epic is deleted, all its children stories are deleted too.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Epic {
    pub name: String,
    pub description: String,
    pub status: Status,
    #[serde(rename = "storyIds")]
    pub story_ids: Vec<u32>,
}

/// A `Story` is a story in the `JiraDatabase`. It is a smaller task that is easier to acheive
/// compared to a story. Each story must have at most one parent `Epic` to associate to.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
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
            status: Status::Open,
        }
    }
}

impl From<String> for Status {
    fn from(value: String) -> Self {
        match value.as_str() {
            "Open" => Self::Open,
            "In Progress" => Self::InProgress,
            "Resolved" => Self::Resolved,
            "Closed" => Self::Closed,
            _ => Self::Open,
        }
    }
}

impl Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Open => "Open",
            Self::InProgress => "In Progress",
            Self::Resolved => "Resolved",
            Self::Closed => "Closed",
        })
    }
}
