#![allow(dead_code)]

use std::rc::Rc;

use anyhow::anyhow;
use itertools::Itertools;
use tabled::{
    builder,
    settings::{self, object::Rows, style::LineText},
};

use crate::{db::JiraDatabase, models::Action};

/// A `Page` is a view that can be drawn on the terminal.
pub trait Page {
    fn draw(&self) -> anyhow::Result<()>;
    fn action_from(&self, input: &str) -> anyhow::Result<Option<Action>>;
}

/// `HomePage` is the first page that a user sees when running the application.
pub struct HomePage {
    pub db: Rc<JiraDatabase>,
}

/// `EpicDetail` is a page with the details of an epic.
pub struct EpicDetail {
    pub epic_id: u32,
    pub db: Rc<JiraDatabase>,
}

/// `StoryDetail` is a page with details of a story.
pub struct StoryDetail {
    pub story_id: u32,
    pub epic_id: u32,
    pub db: Rc<JiraDatabase>,
}

impl Page for HomePage {
    fn draw(&self) -> anyhow::Result<()> {
        let mut builder = builder::Builder::new();
        builder.push_record(["ID", "Name", "Status"]);

        let db = self.db.read()?;
        db.epics.keys().sorted().for_each(|id| {
            let epic = db.epics.get(id).cloned().unwrap();
            builder.push_record([id.to_string(), epic.name, epic.status.to_string()]);
        });
        let table = builder
            .build()
            .with(settings::Style::rounded())
            .with(
                LineText::new(
                    format!("Epics ({})", &db.epics.keys().count()),
                    Rows::first(),
                )
                .offset(2),
            )
            .to_string();

        println!("{}", table);
        println!("(q) quit | (n) new epic | <ID> view epic");
        Ok(())
    }

    fn action_from(&self, input: &str) -> anyhow::Result<Option<Action>> {
        match input {
            "q" => Ok(Some(Action::Exit)),
            "n" => Ok(Some(Action::CreateEpic)),
            other => {
                let epic_id = other
                    .parse::<u32>()
                    .or(Err(anyhow!("invalid action recieved")))?;
                if self.db.read()?.epics.get(&epic_id).is_none() {
                    Ok(None)
                } else {
                    Ok(Some(Action::NavigateToEpicDetail { epic_id }))
                }
            }
        }
    }
}

impl Page for EpicDetail {
    fn draw(&self) -> anyhow::Result<()> {
        let db = self.db.read()?;
        let mut builder = builder::Builder::new();
        builder.push_record(["Name", "Description"]);

        let epic = db.epics.get(&self.epic_id).cloned().unwrap();
        builder.push_record([epic.name, epic.description]);
        let table = builder
            .build()
            .with(settings::Style::rounded())
            .with(
                LineText::new(
                    format!("Epic #{} ({})", &self.epic_id, &epic.status),
                    Rows::first(),
                )
                .offset(2),
            )
            .to_string();
        println!("{}", table);

        let mut story_ids = epic.story_ids.clone();
        if story_ids.is_empty() {
            println!("This epic has no stories.\n");
            println!("(b) back | (u) update | (d) delete | (n) new story | <ID> view story");
            return Ok(());
        }

        let mut builder = builder::Builder::new();
        builder.push_record(["ID", "Name", "Status"]);

        story_ids.sort();
        for id in story_ids {
            let story = db.stories.get(&id).cloned().unwrap();
            builder.push_record([id.to_string(), story.name, story.status.to_string()]);
        }
        let table = builder
            .build()
            .with(settings::Style::rounded())
            .with(
                LineText::new(
                    format!("Stories ({} total)", &epic.story_ids.len()),
                    Rows::first(),
                )
                .offset(2),
            )
            .to_string();

        println!("{}", table);
        println!("(b) back | (u) update | (d) delete | (n) new story | <ID> view story");
        Ok(())
    }

    fn action_from(&self, input: &str) -> anyhow::Result<Option<Action>> {
        match input {
            "b" => Ok(Some(Action::NavigateToPreviousPage)),
            "u" => Ok(Some(Action::UpdateEpicStatus {
                epic_id: self.epic_id,
            })),
            "n" => Ok(Some(Action::CreateStory {
                epic_id: self.epic_id,
            })),
            other => {
                let id = other
                    .parse::<u32>()
                    .or(Err(anyhow!("invalid action recieved")))?;
                if self.db.read()?.stories.get(&id).is_none() {
                    Ok(None)
                } else {
                    Ok(Some(Action::NavigateToStoryDetail {
                        story_id: id,
                        epic_id: self.epic_id,
                    }))
                }
            }
        }
    }
}

impl Page for StoryDetail {
    fn draw(&self) -> anyhow::Result<()> {
        let db = self.db.read()?;
        let mut builder = builder::Builder::new();
        builder.push_record(["Name", "Description"]);

        let story = db.stories.get(&self.story_id).cloned().unwrap();
        builder.push_record([story.name, story.description]);
        let table = builder
            .build()
            .with(settings::Style::rounded())
            .with(
                LineText::new(
                    format!("Story #{} ({})", &self.story_id, &story.status),
                    Rows::first(),
                )
                .offset(2),
            )
            .to_string();

        println!("{}", table);
        println!("(b) back | (u) update | (d) delete");
        Ok(())
    }

    fn action_from(&self, input: &str) -> anyhow::Result<Option<Action>> {
        match input {
            "b" => Ok(Some(Action::NavigateToPreviousPage)),
            "u" => Ok(Some(Action::UpdateStoryStatus {
                story_id: self.story_id,
            })),
            "d" => Ok(Some(Action::DeleteStory {
                story_id: self.story_id,
                epic_id: self.epic_id,
            })),
            _ => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::test_utils::MockDatabase;

    mod home_page {
        use crate::models::Epic;

        use super::*;

        #[test]
        fn draw_should_succeed() {
            let db = Rc::new(JiraDatabase {
                db: Box::new(MockDatabase::new()),
            });
            let page = HomePage { db };
            assert!(page.draw().is_ok());
        }

        #[test]
        fn action_from_quit_action_should_succeed() {
            let db = Rc::new(JiraDatabase {
                db: Box::new(MockDatabase::new()),
            });
            let page = HomePage { db: db.clone() };
            let quit_action = page.action_from("q");
            assert!(quit_action.is_ok());
            assert_eq!(quit_action.unwrap(), Some(Action::Exit));
        }

        #[test]
        fn action_from_new_action_should_succeed() {
            let db = Rc::new(JiraDatabase {
                db: Box::new(MockDatabase::new()),
            });
            let page = HomePage { db: db.clone() };
            let create_action = page.action_from("n");
            assert_eq!(create_action.unwrap(), Some(Action::CreateEpic));
        }

        #[test]
        fn action_from_view_epic_action_should_succeed() {
            let db = Rc::new(JiraDatabase {
                db: Box::new(MockDatabase::new()),
            });
            let page = HomePage { db: db.clone() };

            let epic = Epic::new("Epic 1", "Epic 1 description");
            let epic_id = db.create_epic(&epic).unwrap();
            let view_epic_action = page.action_from(&epic_id.to_string());
            assert!(view_epic_action.is_ok());
            assert_eq!(
                view_epic_action.unwrap(),
                Some(Action::NavigateToEpicDetail { epic_id })
            );
        }

        #[test]
        fn action_from_view_epic_action_should_fail_if_invalid_input() {
            let db = Rc::new(JiraDatabase {
                db: Box::new(MockDatabase::new()),
            });
            let page = HomePage { db: db.clone() };
            let view_epic_action = page.action_from("invalid");
            assert!(view_epic_action.is_err());
        }

        #[test]
        fn action_from_view_epic_action_should_fail_if_invalid_epic_id() {
            let db = Rc::new(JiraDatabase {
                db: Box::new(MockDatabase::new()),
            });
            let page = HomePage { db: db.clone() };

            let invalid_epic_id = 999u32.to_string();
            let view_epic_action = page.action_from(&invalid_epic_id);
            assert!(view_epic_action.is_ok());
            assert!(view_epic_action.unwrap().is_none());
        }
    }

    mod epic_detail {
        use crate::models::{Epic, Story};

        use super::*;

        #[test]
        fn draw_should_succeed() {
            let db = Rc::new(JiraDatabase {
                db: Box::new(MockDatabase::new()),
            });
            let epic_id = db
                .create_epic(&Epic::new("Epic 1", "Epic 1 description"))
                .unwrap();
            let page = EpicDetail { db, epic_id };
            assert!(page.draw().is_ok());
        }

        #[test]
        fn action_from_back_action_should_succeed() {
            let db = Rc::new(JiraDatabase {
                db: Box::new(MockDatabase::new()),
            });
            let epic_id = db
                .create_epic(&Epic::new("Epic 1", "Epic 1 description"))
                .unwrap();
            let page = EpicDetail { db, epic_id };

            let back_action = page.action_from("b");
            assert!(back_action.is_ok());
            assert_eq!(back_action.unwrap(), Some(Action::NavigateToPreviousPage));
        }

        #[test]
        fn action_from_update_action_should_succeed() {
            let db = Rc::new(JiraDatabase {
                db: Box::new(MockDatabase::new()),
            });
            let epic_id = db
                .create_epic(&Epic::new("Epic 1", "Epic 1 description"))
                .unwrap();
            let page = EpicDetail { db, epic_id };

            let update_action = page.action_from("u");
            assert!(update_action.is_ok());
            assert_eq!(
                update_action.unwrap(),
                Some(Action::UpdateEpicStatus { epic_id })
            );
        }

        #[test]
        fn action_from_new_story_action_should_succeed() {
            let db = Rc::new(JiraDatabase {
                db: Box::new(MockDatabase::new()),
            });
            let epic_id = db
                .create_epic(&Epic::new("Epic 1", "Epic 1 description"))
                .unwrap();
            let page = EpicDetail { db, epic_id };

            let new_action = page.action_from("n");
            assert!(new_action.is_ok());
            assert_eq!(new_action.unwrap(), Some(Action::CreateStory { epic_id }));
        }

        #[test]
        fn action_from_view_story_action_should_succeed() {
            let db = Rc::new(JiraDatabase {
                db: Box::new(MockDatabase::new()),
            });
            let epic_id = db
                .create_epic(&Epic::new("Epic 1", "Epic 1 description"))
                .unwrap();
            let story_id = db
                .create_story(&Story::new("Story 1", "Story 1 description"), epic_id)
                .unwrap();
            let page = EpicDetail { db, epic_id };

            let view_story_action = page.action_from(&story_id.to_string());
            assert!(view_story_action.is_ok());
            assert_eq!(
                view_story_action.unwrap(),
                Some(Action::NavigateToStoryDetail { story_id, epic_id })
            );
        }

        #[test]
        fn action_from_view_story_action_should_fail_if_invalid_story_id() {
            let db = Rc::new(JiraDatabase {
                db: Box::new(MockDatabase::new()),
            });
            let epic_id = db
                .create_epic(&Epic::new("Epic 1", "Epic 1 description"))
                .unwrap();
            let page = EpicDetail { db, epic_id };

            let view_story_action = page.action_from("999");
            assert!(view_story_action.is_ok());
            assert!(view_story_action.unwrap().is_none());
        }

        #[test]
        fn action_from_unknown_action_should_fail() {
            let db = Rc::new(JiraDatabase {
                db: Box::new(MockDatabase::new()),
            });
            let epic_id = db
                .create_epic(&Epic::new("Epic 1", "Epic 1 description"))
                .unwrap();
            let page = EpicDetail { db, epic_id };

            let view_story_action = page.action_from("invalid");
            assert!(view_story_action.is_err());
        }
    }

    mod story_detail {
        use crate::models::{Epic, Story};

        use super::*;

        #[test]
        fn draw_should_succeed() {
            let db = Rc::new(JiraDatabase {
                db: Box::new(MockDatabase::new()),
            });
            let epic_id = db
                .create_epic(&Epic::new("Epic 1", "Epic 1 description"))
                .unwrap();
            let story_id = db
                .create_story(&Story::new("Story 1", "Story 1 description"), epic_id)
                .unwrap();
            let page = StoryDetail {
                story_id,
                epic_id,
                db,
            };
            assert!(page.draw().is_ok());
        }

        #[test]
        fn action_from_back_action_should_succeed() {
            let db = Rc::new(JiraDatabase {
                db: Box::new(MockDatabase::new()),
            });
            let epic_id = db
                .create_epic(&Epic::new("Epic 1", "Epic 1 description"))
                .unwrap();
            let story_id = db
                .create_story(&Story::new("Story 1", "Story 1 description"), epic_id)
                .unwrap();
            let page = StoryDetail {
                story_id,
                epic_id,
                db,
            };

            let back_action = page.action_from("b");
            assert!(back_action.is_ok());
            assert_eq!(back_action.unwrap(), Some(Action::NavigateToPreviousPage));
        }

        #[test]
        fn action_from_update_action_should_succeed() {
            let db = Rc::new(JiraDatabase {
                db: Box::new(MockDatabase::new()),
            });
            let epic_id = db
                .create_epic(&Epic::new("Epic 1", "Epic 1 description"))
                .unwrap();
            let story_id = db
                .create_story(&Story::new("Story 1", "Story 1 description"), epic_id)
                .unwrap();
            let page = StoryDetail {
                story_id,
                epic_id,
                db,
            };

            let update_action = page.action_from("u");
            assert!(update_action.is_ok());
            assert_eq!(
                update_action.unwrap(),
                Some(Action::UpdateStoryStatus { story_id })
            );
        }

        #[test]
        fn action_from_delete_action_should_succeed() {
            let db = Rc::new(JiraDatabase {
                db: Box::new(MockDatabase::new()),
            });
            let epic_id = db
                .create_epic(&Epic::new("Epic 1", "Epic 1 description"))
                .unwrap();
            let story_id = db
                .create_story(&Story::new("Story 1", "Story 1 description"), epic_id)
                .unwrap();
            let page = StoryDetail {
                story_id,
                epic_id,
                db,
            };

            let delete_action = page.action_from("d");
            assert!(delete_action.is_ok());
            assert_eq!(
                delete_action.unwrap(),
                Some(Action::DeleteStory { story_id, epic_id })
            );
        }

        #[test]
        fn action_from_unknown_action_should_do_nothing() {
            let db = Rc::new(JiraDatabase {
                db: Box::new(MockDatabase::new()),
            });
            let epic_id = db
                .create_epic(&Epic::new("Epic 1", "Epic 1 description"))
                .unwrap();
            let story_id = db
                .create_story(&Story::new("Story 1", "Story 1 description"), epic_id)
                .unwrap();
            let page = StoryDetail {
                story_id,
                epic_id,
                db,
            };

            let unknown_action = page.action_from("unknown");
            assert!(unknown_action.is_ok());
            assert!(unknown_action.unwrap().is_none());
        }
    }
}
