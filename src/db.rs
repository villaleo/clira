use std::fs;

use anyhow::Result;

use crate::models::{DatabaseState, Epic, Status, Story};

pub struct JiraDatabase {
    db: Box<dyn Database>,
}

#[derive(Clone, Debug)]
pub enum Feature {
    Epic(Epic),
    Story(Story),
}

trait Database {
    fn read(&self) -> Result<DatabaseState>;
    fn write(&self, state: &DatabaseState) -> Result<()>;
}

struct JSONFileDatabase {
    pub file_path: String
}

impl JiraDatabase {
    /// `new` creates a new instance of the `JiraDatabase`. There should be a single instance of
    /// this type to avoid any issues when reading and writing to disk.
    pub fn new(file_path: &str) -> Self {
        Self { db: Box::new(JSONFileDatabase { file_path: file_path.to_string() }) }
    }

    pub fn read(&self) -> Result<DatabaseState> {
        todo!()
    }

    pub fn create(&self, feat: &Feature) -> Result<u32> {
        todo!()
    }

    pub fn update_status(&self, id: u32, status: Status) -> Result<()> {
        todo!()
    }

    pub fn delete(&self, id: u32) -> Result<()> {
        todo!()
    }
}

impl Database for JSONFileDatabase {
    fn read(&self) -> Result<DatabaseState> {
        let data = fs::read_to_string(&self.file_path)?;
        let data: DatabaseState = serde_json::from_str(&data)?;
        Ok(data)
    }

    fn write(&self, state: &DatabaseState) -> Result<()> {
        let data = serde_json::to_string(state)?;
        fs::write(&self.file_path, data)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod database {
        use std::{collections::HashMap, io::Write};

        use crate::models::{Epic, Story};

        use super::*;

        #[test]
        fn read_should_fail_with_invalid_path() {
            let db = JSONFileDatabase { file_path: "invalid".to_owned() };
            assert!(db.read().is_err());
        }

        #[test]
        fn read_should_fail_with_invalid_json() {
            let mut file = tempfile::NamedTempFile::new()
                .expect("failed to create a temporary file");
            let malformed_data = r#"{ "lastItemId": null, "epics": {} "stories": {} }"#;
            write!(file, "{}", malformed_data)
                .expect("failed to write to temporary file");

            let file_path = file.path().to_str()
                .expect("failed to cast temporary file path to str");
            let db = JSONFileDatabase { file_path: file_path.to_string() };
            let read = db.read();
            assert!(read.is_err());
        }

        #[test]
        fn read_should_parse_json_file() {
            let mut file = tempfile::NamedTempFile::new()
                .expect("failed to create a temporary file");
            let data = r#"{ "lastItemId": null, "epics": {}, "stories": {} }"#;
            write!(file, "{}", data)
                .expect("failed to write to temporary file");

            let file_path = file.path().to_str()
                .expect("failed to cast temporary file path to str");
            let db = JSONFileDatabase { file_path: file_path.to_string() };
            let read = db.read();
            assert!(read.is_ok());
        }

        #[test]
        fn write_should_write_to_file() {
            let mut file = tempfile::NamedTempFile::new()
                .expect("failed to create a temporary file");
            let data = r#"{ "lastItemId": null, "epics": {}, "stories": {} }"#;
            write!(file, "{}", data)
                .expect("failed to write to temporary file");

            let file_path = file.path().to_str()
                .expect("failed to cast temporary file path to str");
            let db = JSONFileDatabase { file_path: file_path.to_string() };

            let story = Story::new("Create Unit Tests", "Write unit tests for the project.");
            let mut stories = HashMap::<u32, Story>::new();
            stories.insert(0, story);
            let epic = Epic::new("Jira CLI", "Create a CLI interface for Jira.");
            let mut epics = HashMap::<u32, Epic>::new();
            epics.insert(1, epic);

            let state = DatabaseState { last_item_id: Some(1u32), epics, stories };
            assert!(db.write(&state).is_ok());
            let data = db.read()
                .expect("failed to read data from database");
            assert_eq!(data, state);
        }
    }
}

