#![allow(dead_code)]

use std::{collections::HashMap, fs};

use anyhow::{anyhow, bail, Result};

use crate::models::{DatabaseState, Epic, Status, Story};

/// `JiraDatabase` is the main database for the application to interact with. There should be at
/// most one instance of this type. Instances need not be mutable.
pub struct JiraDatabase {
    pub db: Box<dyn Database>,
}

/// `Database` outlines the main functionalities of a database. Use `read` to fetch the current
/// state of the database.
pub trait Database {
    fn read(&self) -> Result<DatabaseState>;
    fn write(&self, state: &DatabaseState) -> Result<()>;
}

struct JSONFileDatabase {
    pub file_path: String,
}

impl JiraDatabase {
    /// `new` creates a new instance of the `JiraDatabase`. There should be a single instance of
    /// this type to avoid any issues when reading and writing to disk. Returns `JiraDatabase`
    /// wrapped in `Result`.
    ///
    /// `Err` means there was a problem initializing the database.
    pub fn new(file_path: &str) -> Result<Self> {
        let db = JSONFileDatabase {
            file_path: file_path.to_string(),
        };
        if let Ok(state) = db.read() {
            db.write(&state)?;
        } else {
            db.write(&DatabaseState {
                last_item_id: None,
                epics: HashMap::new(),
                stories: HashMap::new(),
            })?;
        }
        Ok(Self { db: Box::new(db) })
    }

    /// `read` reads the data from the database and returns a `DatabaseState` wrapped in a
    /// `Result`.
    ///
    /// `Err` means there was a problem reading from the underlying database.
    pub fn read(&self) -> Result<DatabaseState> {
        let state = self.db.read()?;
        Ok(state)
    }

    /// `create_epic` writes a new epic to the database. Returns the epic's corresponding id
    /// wrapped in a `Result`.
    ///
    /// `Err` will explain the cause, but may be for one of the following reasons:
    ///   - There was a problem reading from the database
    ///   - There was a problem writing to the database
    pub fn create_epic(&self, epic: &Epic) -> Result<u32> {
        let mut state = self.read()?;
        let id = if let Some(prev_id) = state.last_item_id {
            prev_id + 1u32
        } else {
            0u32
        };

        state.last_item_id = Some(id);
        state.epics.insert(id, epic.clone());
        self.db.write(&state)?;
        Ok(id)
    }

    /// `create_story` adds a new story to the epic `epic_id` and writes to the database. Returns
    /// the story's corresponding id wrapped in a `Result`.
    ///
    /// `Err` will explain the cause, but may be for one of the following reasons:
    ///   - There was a problem reading from the database
    ///   - An epic does not exist for the input parameter `epic_id`
    ///   - There was a problem writing to the database
    pub fn create_story(&self, story: &Story, epic_id: u32) -> Result<u32> {
        let mut state = self.read()?;
        let id = if let Some(prev_id) = state.last_item_id {
            prev_id + 1u32
        } else {
            0u32
        };

        let mut epic = state
            .epics
            .get(&epic_id)
            .ok_or(anyhow!("no epic found for id {}", epic_id))
            .cloned()?;
        epic.story_ids.push(id);
        state.epics.insert(epic_id, epic);

        state.last_item_id = Some(id);
        state.stories.insert(id, story.clone());
        self.db.write(&state)?;
        Ok(id)
    }

    /// `update_epic_name` updates the name of the epic `id`. Returns `Err` if epic
    /// was not found or if there was an error reading/writinig to the database.
    pub fn update_epic_name(&self, id: u32, name: &str) -> Result<()> {
        let mut state = self.read()?;
        if let Some(epic) = state.epics.get(&id) {
            let mut epic = epic.clone();
            epic.name = name.to_string();
            state.epics.insert(id, epic);
            self.db.write(&state)?;
        } else {
            bail!("no epic found for id {}", id)
        }
        Ok(())
    }

    /// `update_epic_description` updates the description of the epic `id`. Returns
    /// `Err` if epic was not found or if there was an error reading/writinig to
    /// the database.
    pub fn update_epic_description(&self, id: u32, description: &str) -> Result<()> {
        let mut state = self.read()?;
        if let Some(epic) = state.epics.get(&id) {
            let mut epic = epic.clone();
            epic.description = description.to_string();
            state.epics.insert(id, epic);
            self.db.write(&state)?;
        } else {
            bail!("no epic found for id {}", id)
        }
        Ok(())
    }

    /// `update_epic_status` updates the status of the epic `id` to the new status `status`. Returns
    /// an empty tuple wrapped in a `Result`.
    ///
    /// `Err` will explain the cause, but may be for one of the following reasons:
    ///   - There was a problem reading from the database
    ///   - An epic does not exist for the input parameter `id`
    ///   - There was a problem writing to the database
    pub fn update_epic_status(&self, id: u32, status: Status) -> Result<()> {
        let mut state = self.read()?;
        let mut epic = state
            .epics
            .get(&id)
            .ok_or(anyhow!("no epic found for id {}", id))
            .cloned()?;
        epic.status = status;

        state.epics.insert(id, epic);
        self.db.write(&state)?;
        Ok(())
    }

    /// `update_story_name` updates the name of the story `id`. Returns `Err` if the
    /// story was not found or if there was an error reading/writing to the database.
    pub fn update_story_name(&self, id: u32, name: &str) -> Result<()> {
        let mut state = self.read()?;
        if let Some(story) = state.stories.get(&id) {
            let mut story = story.clone();
            story.name = name.to_string();
            state.stories.insert(id, story);
            self.db.write(&state)?;
            Ok(())
        } else {
            bail!("no story found for id {}", id)
        }
    }

    /// `update_story_description` updates the description of the story `id`. Returns
    /// `Err` if the story was not found or if there was an error reading/writing to
    /// the database.
    pub fn update_story_description(&self, id: u32, description: &str) -> Result<()> {
        let mut state = self.read()?;
        if let Some(story) = state.stories.get(&id) {
            let mut story = story.clone();
            story.description = description.to_string();
            state.stories.insert(id, story);
            self.db.write(&state)?;
            Ok(())
        } else {
            bail!("no story found for id {}", id)
        }
    }

    /// `update_story_status` updates the status of the `id` to the new status `status`. Returns
    /// an empty tuple wrapped in a `Result`.
    ///
    /// `Err` will explain the cause, but may be for one of the following reasons:
    ///   - There was a problem reading from the database
    ///   - An story does not exist for the input parameter `id`
    ///   - There was a problem writing to the database
    pub fn update_story_status(&self, id: u32, status: Status) -> Result<()> {
        let mut state = self.read()?;
        let mut story = state
            .stories
            .get(&id)
            .ok_or(anyhow!("no story found for id {}", id))
            .cloned()?;
        story.status = status;

        state.stories.insert(id, story);
        self.db.write(&state)?;
        Ok(())
    }

    /// `delete_epic` deletes the epic corresponding to `id`. Returns an empty tuple wrapped in a
    /// `Result`.
    ///
    /// `Err` will explain the cause, but may be for one of the following reasons:
    ///   - There was a problem reading from the database
    ///   - An epic does not exist for the input parameter `id`
    ///   - There was a problem writing to the database
    pub fn delete_epic(&self, id: u32) -> Result<()> {
        let mut state = self.read()?;
        let _ = state
            .epics
            .get(&id)
            .ok_or(anyhow!("no epic found for id {}", id))?;

        state.epics.remove(&id);
        self.db.write(&state)?;
        Ok(())
    }

    /// `delete_story` deletes the story corresponding to `id`. Returns an empty tuple wrapped in a
    /// `Result`.
    ///
    /// `Err` will explain the cause, but may be for one of the following reasons:
    ///   - There was a problem reading from the database
    ///   - An epic does not exist for the input parameter `id`
    ///   - There was a problem writing to the database
    pub fn delete_story(&self, story_id: u32, epic_id: u32) -> Result<()> {
        let mut state = self.read()?;
        let mut epic = state
            .epics
            .get(&epic_id)
            .ok_or(anyhow!("no epic found for id {}", &epic_id))?
            .clone();

        let (idx, _) = epic
            .story_ids
            .iter()
            .enumerate()
            .find(|(_, id)| *id == &story_id)
            .ok_or(anyhow!("no story found for id {}", &story_id))?;
        epic.story_ids.remove(idx);

        state.epics.insert(epic_id, epic);
        state.stories.remove(&story_id);
        self.db.write(&state)?;
        Ok(())
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

/// `test_utils` contains utilities used for testing.
pub mod test_utils {
    use std::{cell::RefCell, collections::HashMap};

    use super::*;

    /// `MockDatabase` is a wrapper over the state of a database. It uses the interior
    /// mutability design pattern to keep `JiraDatabase`'s immutability state.
    pub struct MockDatabase {
        last_written_state: RefCell<DatabaseState>,
    }

    impl MockDatabase {
        /// `new` returns an instance of `MockDatabase` initialized and ready to use.
        pub fn new() -> Self {
            Self {
                last_written_state: RefCell::new(DatabaseState {
                    last_item_id: None,
                    epics: HashMap::new(),
                    stories: HashMap::new(),
                }),
            }
        }
    }

    impl Database for MockDatabase {
        fn read(&self) -> Result<DatabaseState> {
            let state = self.last_written_state.borrow().clone();
            Ok(state)
        }

        fn write(&self, state: &DatabaseState) -> Result<()> {
            *self.last_written_state.borrow_mut() = state.clone();
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod jira {
        use self::test_utils::MockDatabase;

        use super::*;

        #[test]
        fn create_epic_should_succeed() {
            let db = JiraDatabase {
                db: Box::new(MockDatabase::new()),
            };
            let epic = Epic::new("Epic 1", "Epic description 1");
            assert!(db.create_epic(&epic).is_ok());
        }

        #[test]
        fn create_story_should_error_on_invalid_epic_id() {
            let db = JiraDatabase {
                db: Box::new(MockDatabase::new()),
            };
            let story = Story::new("Story 1", "Story description 1");

            let invalid_epic_id = 999u32;
            let res = db.create_story(&story, invalid_epic_id);
            assert!(res.is_err());
        }

        #[test]
        fn create_story_should_succeed() {
            let db = JiraDatabase {
                db: Box::new(MockDatabase::new()),
            };

            let epic = Epic::new("Epic 1", "Epic 1 description");
            let epic_id = db.create_epic(&epic).unwrap();
            db.create_epic(&epic).unwrap();

            let story = Story::new("Story 1", "Story 1 description");
            let res = db.create_story(&story, epic_id);
            assert!(res.is_ok());
        }

        #[test]
        fn delete_epic_should_error_on_invalid_epic_id() {
            let db = JiraDatabase {
                db: Box::new(MockDatabase::new()),
            };
            let invalid_epic_id = 999u32;
            let res = db.delete_epic(invalid_epic_id);
            assert!(res.is_err());
        }

        #[test]
        fn delete_epic_should_succeed() {
            let db = JiraDatabase {
                db: Box::new(MockDatabase::new()),
            };
            let epic = Epic::new("Epic 1 ", "Epic 1 description");
            let id = db.create_epic(&epic).unwrap();
            assert!(db.delete_epic(id).is_ok());
        }

        #[test]
        fn delete_story_should_error_on_story_not_found_in_epic() {
            let db = JiraDatabase {
                db: Box::new(MockDatabase::new()),
            };

            let epic = Epic::new("Epic 1", "Epic 1 description");
            let epic_id = db.create_epic(&epic).unwrap();

            let invalid_story_id = 999u32;
            assert!(db.delete_story(invalid_story_id, epic_id).is_err());
        }

        #[test]
        fn delete_story_should_error_on_invalid_story_id() {
            let db = JiraDatabase {
                db: Box::new(MockDatabase::new()),
            };

            let epic = Epic::new("Epic 1", "Epic 1 description");
            let epic_id = db.create_epic(&epic).unwrap();

            let story = Story::new("Story 1", "Story 1 description");
            let story_id = db.create_story(&story, epic_id).unwrap();
            let invalid_epic_id = 999u32;
            assert!(db.delete_story(story_id, invalid_epic_id).is_err());
        }

        #[test]
        fn delete_story_should_succeed() {
            let db = JiraDatabase {
                db: Box::new(MockDatabase::new()),
            };

            let epic = Epic::new("Epic 1", "Epic 1 description");
            let epic_id = db.create_epic(&epic).unwrap();

            let story = Story::new("Story 1", "Story 1 description");
            let story_id = db.create_story(&story, epic_id).unwrap();

            assert!(db.delete_story(story_id, epic_id).is_ok());
        }

        #[test]
        fn update_epic_status_should_error_on_invalid_epic_id() {
            let db = JiraDatabase {
                db: Box::new(MockDatabase::new()),
            };
            let invalid_epic_id = 999u32;
            assert!(db
                .update_epic_status(invalid_epic_id, Status::InProgress)
                .is_err());
        }

        #[test]
        fn update_epic_status_should_succeed() {
            let db = JiraDatabase {
                db: Box::new(MockDatabase::new()),
            };
            let epic = Epic::new("Epic 1", "Epic 1 description");
            let epic_id = db.create_epic(&epic).unwrap();
            assert!(db.update_epic_status(epic_id, Status::InProgress).is_ok());
        }

        #[test]
        fn update_story_status_should_error_on_invalid_story_id() {
            let db = JiraDatabase {
                db: Box::new(MockDatabase::new()),
            };
            let invalid_story_id = 999u32;
            assert!(db
                .update_story_status(invalid_story_id, Status::InProgress)
                .is_err());
        }

        #[test]
        fn update_story_status_should_succeed() {
            let db = JiraDatabase {
                db: Box::new(MockDatabase::new()),
            };

            let epic = Epic::new("Epic 1", "Epic 1 description");
            let epic_id = db.create_epic(&epic).unwrap();

            let story = Story::new("Story 1", "Story 1 description");
            let story_id = db.create_story(&story, epic_id).unwrap();
            assert!(db.update_story_status(story_id, Status::InProgress).is_ok());
        }
    }

    mod database {
        use std::{collections::HashMap, io::Write};

        use crate::models::{Epic, Story};

        use super::*;

        #[test]
        fn read_should_fail_with_invalid_path() {
            let db = JSONFileDatabase {
                file_path: "invalid".to_owned(),
            };
            assert!(db.read().is_err());
        }

        #[test]
        fn read_should_fail_with_invalid_json() {
            let mut file = tempfile::NamedTempFile::new().unwrap();
            let malformed_data = r#"{ "lastItemId": null, "epics": {} "stories": {} }"#;
            write!(file, "{}", malformed_data).unwrap();

            let file_path = file.path().to_str().unwrap();
            let db = JSONFileDatabase {
                file_path: file_path.to_string(),
            };
            assert!(db.read().is_err());
        }

        #[test]
        fn read_should_parse_json_file() {
            let mut file = tempfile::NamedTempFile::new().unwrap();
            let data = r#"{ "lastItemId": null, "epics": {}, "stories": {} }"#;
            write!(file, "{}", data).unwrap();

            let file_path = file.path().to_str().unwrap();
            let db = JSONFileDatabase {
                file_path: file_path.to_string(),
            };
            assert!(db.read().is_ok());
        }

        #[test]
        fn write_should_write_to_file() {
            let mut file = tempfile::NamedTempFile::new().unwrap();
            let data = r#"{ "lastItemId": null, "epics": {}, "stories": {} }"#;
            write!(file, "{}", data).unwrap();

            let file_path = file.path().to_str().unwrap();
            let db = JSONFileDatabase {
                file_path: file_path.to_string(),
            };

            let story = Story::new("Story 1", "Story 1 description");
            let mut stories = HashMap::<u32, Story>::new();
            stories.insert(0, story);
            let epic = Epic::new("Epic 1", "Epic 1 description");
            let mut epics = HashMap::<u32, Epic>::new();
            epics.insert(1, epic);

            let state = DatabaseState {
                last_item_id: Some(1u32),
                epics,
                stories,
            };
            assert!(db.write(&state).is_ok());
            assert_eq!(db.read().unwrap(), state);
        }
    }
}
