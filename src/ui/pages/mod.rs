pub mod prompts;

use std::{any::Any, rc::Rc};

use anyhow::anyhow;
use itertools::Itertools;
use tabled::{
    builder::{self, Builder},
    settings::{
        self,
        object::{Columns, Rows, Segment},
        style::LineText,
        Alignment, Format,
    },
};

use crate::{
    db::JiraDatabase,
    models::Action,
    utils::{color_for_table_header, color_table_column, constrain_text, read_input},
};

/// A `Page` is a view that can be drawn on the terminal.
pub trait Page {
    /// `draw` prints the page to the `stdout`.
    fn draw(&self) -> anyhow::Result<()>;
    /// `draw_menu` draws a menu to the page with all the actions available
    /// to the user.
    fn draw_menu(&self);
    /// `action_from` returns an action, depending on the `input`.
    fn action_from(&self, input: &str) -> anyhow::Result<Option<Action>>;
    /// `as_any` is used to prepare to downcast a trait object to a concrete type.
    fn as_any(&self) -> &dyn Any;
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

pub const MAX_NAME_LENGTH: usize = 30;
pub const MAX_DESCRIPTION_LENGTH: usize = 55;

impl Page for HomePage {
    fn draw(&self) -> anyhow::Result<()> {
        let db = self.db.read()?;
        if db.epics.is_empty() {
            println!("\n  There are no epics. Create a new epic with `n`.");
            self.draw_menu();
            return Ok(());
        }

        let mut builder = builder::Builder::new();
        builder.push_record(["ID", "Name", "Status"]);

        for id in db.epics.keys().sorted() {
            let epic = &db.epics[id];
            builder.push_record([id.to_string(), epic.name.clone(), epic.status.to_string()]);
        }

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
            .modify(Columns::single(2), Format::content(color_table_column))
            .to_string();

        println!("{}", table);
        self.draw_menu();
        Ok(())
    }

    fn draw_menu(&self) {
        let menu = into_table(&["(q) quit", "(n) new epic", "<ID> view epic"]);
        println!("\n\n{}\n\nEnter command:", menu);
    }

    fn action_from(&self, input: &str) -> anyhow::Result<Option<Action>> {
        match input {
            "q" => Ok(Some(Action::Exit)),
            "n" => Ok(Some(Action::CreateEpic)),
            other => {
                if let Ok(epic_id) = other.parse::<u32>() {
                    if self.db.read()?.epics.contains_key(&epic_id) {
                        return Ok(Some(Action::NavigateToEpicDetail { epic_id }));
                    }
                }
                Ok(None)
            }
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Page for EpicDetail {
    fn draw(&self) -> anyhow::Result<()> {
        let db = self.db.read()?;
        let mut builder = builder::Builder::new();
        builder.push_record(["Name", "Description"]);

        let epic = db
            .epics
            .get(&self.epic_id)
            .ok_or_else(|| anyhow!("could not find epic"))?;
        builder.push_record([
            &constrain_text(&epic.name, MAX_NAME_LENGTH),
            &constrain_text(&epic.description, MAX_DESCRIPTION_LENGTH),
        ]);

        let table = builder
            .build()
            .with(settings::Style::rounded())
            .with(LineText::new(format!("Epic #{} (", &self.epic_id), Rows::first()).offset(2))
            .with(
                LineText::new(format!("{}", &epic.status), Rows::first())
                    .color(color_for_table_header(&epic.status.to_string()))
                    .offset(2 + format!("Epic #{} (", &self.epic_id).len()),
            )
            .with(
                LineText::new(")", Rows::first())
                    .offset(2 + format!("Epic #{} ({}", &self.epic_id, &epic.status).len()),
            )
            .to_string();
        println!("{}", table);

        let mut story_ids = epic.story_ids.clone();
        if story_ids.is_empty() {
            println!("\n  This epic has no stories.");
            self.draw_menu();
            return Ok(());
        }

        let mut builder = builder::Builder::new();
        builder.push_record(["ID", "Name", "Status"]);

        story_ids.sort();
        for id in story_ids {
            let story = db
                .stories
                .get(&id)
                .ok_or_else(|| anyhow!("could not find story"))?;
            builder.push_record([
                id.to_string(),
                constrain_text(story.name.as_str(), MAX_NAME_LENGTH),
                constrain_text(&story.status.to_string(), MAX_DESCRIPTION_LENGTH),
            ]);
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
            .modify(Columns::single(2), Format::content(color_table_column))
            .to_string();

        println!("\n{}", table);
        self.draw_menu();
        Ok(())
    }

    fn draw_menu(&self) {
        let menu = into_table(&[
            "(b) back",
            "(u) update",
            "(d) delete",
            "(n) new story",
            "<ID> view story",
        ]);
        println!("\n\n{}\n\nEnter command:", menu);
    }

    fn action_from(&self, input: &str) -> anyhow::Result<Option<Action>> {
        match input {
            "b" => Ok(Some(Action::NavigateToPreviousPage)),
            "u" => Ok(update_epic(self.epic_id)),
            "d" => Ok(Some(Action::DeleteEpic {
                epic_id: self.epic_id,
            })),
            "n" => Ok(Some(Action::CreateStory {
                epic_id: self.epic_id,
            })),
            other => {
                if let Ok(story_id) = other.parse::<u32>() {
                    if self.db.read()?.stories.contains_key(&story_id) {
                        return Ok(Some(Action::NavigateToStoryDetail {
                            story_id,
                            epic_id: self.epic_id,
                        }));
                    }
                }
                Ok(None)
            }
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Page for StoryDetail {
    fn draw(&self) -> anyhow::Result<()> {
        let db = self.db.read()?;
        let mut builder = builder::Builder::new();
        builder.push_record(["Name", "Description"]);

        let story = db
            .stories
            .get(&self.story_id)
            .ok_or_else(|| anyhow!("could not find story"))?;
        builder.push_record([
            constrain_text(&story.name, MAX_NAME_LENGTH),
            constrain_text(&story.description, MAX_DESCRIPTION_LENGTH),
        ]);

        let table = builder
            .build()
            .with(settings::Style::rounded())
            .with(LineText::new(format!("Story #{} (", &self.story_id), Rows::first()).offset(2))
            .with(
                LineText::new(format!("{}", &story.status), Rows::first())
                    .color(color_for_table_header(&story.status.to_string()))
                    .offset(2 + format!("Story #{} (", &self.story_id).len()),
            )
            .with(
                LineText::new(")", Rows::first())
                    .offset(2 + format!("Story #{} ({}", &self.story_id, &story.status).len()),
            )
            .to_string();

        println!("{}", table);
        self.draw_menu();
        Ok(())
    }

    fn draw_menu(&self) {
        let menu = into_table(&["(b) back", "(u) update", "(d) delete"]);
        println!("\n\n{}\n\nEnter command:", menu);
    }

    fn action_from(&self, input: &str) -> anyhow::Result<Option<Action>> {
        match input {
            "b" => Ok(Some(Action::NavigateToPreviousPage)),
            "u" => Ok(update_story(self.story_id)),
            "d" => Ok(Some(Action::DeleteStory {
                story_id: self.story_id,
                epic_id: self.epic_id,
            })),
            _ => Ok(None),
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

fn into_table(opts: &[&str]) -> String {
    let mut builder = Builder::new();
    builder.push_record(opts.iter().map(|s| s.to_owned()));
    builder
        .build()
        .with(settings::Modify::new(Segment::all()).with(Alignment::center()))
        .with(settings::Style::modern_rounded())
        .to_string()
}

fn update_epic(epic_id: u32) -> Option<Action> {
    println!("Update which field?\n\t1 - Name\n\t2 - Description\n\t3 - Status");
    println!("(x) cancel");
    match read_input().as_str() {
        "1" => Some(Action::UpdateEpicName { epic_id }),
        "2" => Some(Action::UpdateEpicDescription { epic_id }),
        "3" => Some(Action::UpdateEpicStatus { epic_id }),
        _ => None,
    }
}

fn update_story(story_id: u32) -> Option<Action> {
    println!("Update which field?\n\t1 - Name\n\t2 - Description\n\t3 - Status");
    println!("(x) cancel");
    match read_input().as_str() {
        "1" => Some(Action::UpdateStoryName { story_id }),
        "2" => Some(Action::UpdateStoryDescription { story_id }),
        "3" => Some(Action::UpdateStoryStatus { story_id }),
        _ => None,
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
            assert!(view_epic_action.is_ok());
            assert!(view_epic_action.unwrap().is_none());
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
        use crate::{
            models::{Epic, Status, Story},
            ui::navigator::{test_utils::MockNavigator, NavigationManager},
        };

        use self::prompts::Prompt;

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
            let epic_id = db.create_epic(&Epic::new("name", "description")).unwrap();
            let mut nav = MockNavigator::new(db);

            let mut prompts = Prompt::new();
            prompts.update_name = Box::new(|| "new name".to_string());
            nav.set_prompts(prompts);
            let res = nav.dispatch_action(Action::UpdateEpicName { epic_id });
            assert!(res.is_ok());
            assert_eq!(
                nav.state
                    .clone()
                    .last_written_state
                    .borrow()
                    .epics
                    .get(&epic_id)
                    .unwrap()
                    .name,
                "new name".to_string()
            );

            let mut prompts = Prompt::new();
            prompts.update_description = Box::new(|| "new description".to_string());
            nav.set_prompts(prompts);
            let res = nav.dispatch_action(Action::UpdateEpicDescription { epic_id });
            assert!(res.is_ok());
            assert_eq!(
                nav.state
                    .clone()
                    .last_written_state
                    .borrow()
                    .epics
                    .get(&epic_id)
                    .unwrap()
                    .description,
                "new description".to_string()
            );

            let mut prompts = Prompt::new();
            prompts.update_status = Box::new(|| Some(Status::Closed));
            nav.set_prompts(prompts);
            let res = nav.dispatch_action(Action::UpdateEpicStatus { epic_id });
            assert!(res.is_ok());
            assert_eq!(
                nav.state
                    .clone()
                    .last_written_state
                    .borrow()
                    .epics
                    .get(&epic_id)
                    .unwrap()
                    .status,
                Status::Closed
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
            assert!(view_story_action.is_ok());
            assert!(view_story_action.unwrap().is_none());
        }
    }

    mod story_detail {
        use story_detail::tests::prompts::Prompt;

        use crate::{
            models::{Epic, Status, Story},
            ui::navigator::{test_utils::MockNavigator, NavigationManager},
        };

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

            let epic_id = db.create_epic(&Epic::new("name", "description")).unwrap();
            let story_id = db
                .create_story(&Story::new("name", "description"), epic_id)
                .unwrap();
            let mut nav = MockNavigator::new(db);

            let mut prompts = Prompt::new();
            prompts.update_name = Box::new(|| "new name".to_string());
            nav.set_prompts(prompts);
            let res = nav.dispatch_action(Action::UpdateStoryName { story_id });
            assert!(res.is_ok());
            assert_eq!(
                nav.state
                    .clone()
                    .last_written_state
                    .borrow()
                    .stories
                    .get(&story_id)
                    .unwrap()
                    .name,
                "new name".to_string()
            );

            let mut prompts = Prompt::new();
            prompts.update_description = Box::new(|| "new description".to_string());
            nav.set_prompts(prompts);
            let res = nav.dispatch_action(Action::UpdateStoryDescription { story_id });
            assert!(res.is_ok());
            assert_eq!(
                nav.state
                    .clone()
                    .last_written_state
                    .borrow()
                    .stories
                    .get(&story_id)
                    .unwrap()
                    .description,
                "new description".to_string()
            );

            let mut prompts = Prompt::new();
            prompts.update_status = Box::new(|| Some(Status::Closed));
            nav.set_prompts(prompts);
            let res = nav.dispatch_action(Action::UpdateStoryStatus { story_id });
            assert!(res.is_ok());
            assert_eq!(
                nav.state
                    .clone()
                    .last_written_state
                    .borrow()
                    .stories
                    .get(&story_id)
                    .unwrap()
                    .status,
                Status::Closed
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
