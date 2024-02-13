use std::rc::Rc;

use anyhow::anyhow;

use crate::{
    db::JiraDatabase,
    models::{Action, Status},
    ui::pages::{prompts::Prompt, EpicDetail, HomePage, Page, StoryDetail},
};

pub trait NavigationManager {
    /// `current_page` gets the current page that is rendered on the stack.
    fn current_page(&self) -> Option<&dyn Page>;
    /// `dispatch_action` commits the `action` to the database.
    fn dispatch_action(&mut self, action: Action) -> anyhow::Result<()>;
}

/// `Navigator` manages the navigation stack between different pages.
pub struct Navigator {
    pages: Vec<Box<dyn Page>>,
    prompts: Prompt,
    db: Rc<JiraDatabase>,
}

/// A Feature represents the different types of features in the program.
/// It is used to avoid repeating logic for the different types of
/// features.
enum Feature {
    Epic(u32),
    Story(u32),
}

impl Navigator {
    /// `new` returns a new instance of `Navigator` ready to use.
    pub fn new(db: Rc<JiraDatabase>) -> Self {
        Self {
            pages: vec![Box::new(HomePage { db: db.clone() })],
            prompts: Prompt::new(),
            db: db.clone(),
        }
    }

    #[allow(dead_code)]
    // `page_count` is used for testing. If `warn(dead_code)` is enabled, then cargo check will incorrectly
    // report unused code.
    fn page_count(&self) -> usize {
        self.pages.len()
    }

    #[allow(dead_code)]
    // `set_prompts` is used for testing. If `warn(dead_code)` is enabled, then cargo check will incorrectly
    // report unused code.
    fn set_prompts(&mut self, prompt: Prompt) {
        self.prompts = prompt;
    }

    /// `auto_update_epic_status` updates an Epic's status based on its children Stories.
    /// Epics are updated based on the `feature`'s id. The status of the Epic is updated
    /// based on the following conditions, where the higher conditions have higher
    /// precedence:
    /// - All stories Closed => Closed
    /// - All stories Resolved or Closed => Resolved
    /// - All stories Open => Open
    /// - Otherwise => In Progress
    fn auto_update_epic_status(&self, feat: Feature) -> anyhow::Result<()> {
        let state = self.db.read()?;
        let (epic_id, epic) = match feat {
            Feature::Epic(ref epic_id) => (
                epic_id,
                state.epics.get(epic_id).ok_or(anyhow!("epic not found"))?,
            ),
            Feature::Story(story_id) => state
                .epics
                .iter()
                .find(|(_, epic)| epic.story_ids.contains(&story_id))
                .ok_or(anyhow!("epic not found"))?,
        };
        let stories: Vec<_> = epic
            .story_ids
            .iter()
            .filter_map(|id| state.stories.get(id))
            .collect();
        if stories.is_empty() {
            self.db.update_epic_status(*epic_id, Status::Open)?;
            return Ok(());
        }
        let status = if stories
            .iter()
            .all(|story| matches!(story.status, Status::Closed))
        {
            Status::Closed
        } else if stories
            .iter()
            .all(|story| matches!(story.status, Status::Resolved | Status::Closed))
        {
            Status::Resolved
        } else if stories
            .iter()
            .all(|story| matches!(story.status, Status::Open))
        {
            Status::Open
        } else {
            Status::InProgress
        };
        self.db.update_epic_status(*epic_id, status)?;
        Ok(())
    }
}

impl NavigationManager for Navigator {
    fn current_page(&self) -> Option<&dyn Page> {
        self.pages.last().map(|page| page.as_ref())
    }

    fn dispatch_action(&mut self, action: Action) -> anyhow::Result<()> {
        match action {
            Action::NavigateToEpicDetail { epic_id } => {
                let page = Box::new(EpicDetail {
                    epic_id,
                    db: self.db.clone(),
                });
                self.pages.push(page);
            }
            Action::NavigateToStoryDetail { story_id, epic_id } => {
                let page = Box::new(StoryDetail {
                    story_id,
                    epic_id,
                    db: self.db.clone(),
                });
                self.pages.push(page);
            }
            Action::NavigateToPreviousPage => {
                self.pages.pop();
            }
            Action::CreateEpic => {
                let epic = (self.prompts.create_epic)();
                self.db.create_epic(&epic)?;
            }
            Action::CreateStory { epic_id } => {
                let story = (self.prompts.create_story)();
                self.db.create_story(&story, epic_id)?;
            }
            Action::UpdateEpicName { epic_id } => {
                let name = (self.prompts.update_name)();
                self.db.update_epic_name(epic_id, &name)?;
            }
            Action::UpdateEpicDescription { epic_id } => {
                let description = (self.prompts.update_description)();
                self.db.update_epic_description(epic_id, &description)?;
            }
            Action::UpdateEpicStatus { epic_id } => {
                if let Some(status) = (self.prompts.update_status)() {
                    self.db.update_epic_status(epic_id, status)?;
                }
            }
            Action::UpdateStoryName { story_id } => {
                let name = (self.prompts.update_name)();
                self.db.update_story_name(story_id, &name)?;
            }
            Action::UpdateStoryDescription { story_id } => {
                let description = (self.prompts.update_description)();
                self.db.update_story_description(story_id, &description)?;
            }
            Action::UpdateStoryStatus { story_id } => {
                if let Some(status) = (self.prompts.update_status)() {
                    self.db.update_story_status(story_id, status)?;
                    self.auto_update_epic_status(Feature::Story(story_id))?;
                }
            }
            Action::DeleteEpic { epic_id } => {
                if (self.prompts.delete_epic)() {
                    self.db.delete_epic(epic_id)?;
                    self.pages.pop();
                }
            }
            Action::DeleteStory { story_id, epic_id } => {
                if (self.prompts.delete_story)() {
                    self.db.delete_story(story_id, epic_id)?;
                    self.auto_update_epic_status(Feature::Epic(epic_id))?;
                    self.pages.pop();
                }
            }
            Action::Exit => self.pages.clear(),
        }
        Ok(())
    }
}

pub mod test_utils {
    use std::cell::RefCell;

    use crate::db::test_utils::MockDatabase;

    use super::*;

    /// `MockNavigator` is an implementation of `NavigationManager` with public members,
    /// used for testing. This type used to simulates and test `Navigator`.
    pub struct MockNavigator {
        pub pages: Vec<Box<dyn Page>>,
        pub prompts: Prompt,
        pub db: Rc<JiraDatabase>,
        pub state: Rc<MockDatabase>,
    }

    impl MockNavigator {
        /// `new` creates a new instance of `MockNavigator`. Pass an instance of `JiraDatabase`
        /// instantiated like below:
        /// ```
        /// let db = Rc::new(JiraDatabase {
        ///     db: Box::new(MockDatabase::new()),
        /// });
        /// ```
        /// NOTE: If you do multiple tests in a single `test` function, you will need to shadow the `db`
        /// variable to create a new instance of `Rc<JiraDatabase>`,
        ///
        /// This instance will be moved into `new` and be consumed by the function. If you need
        /// to access `db` again, you will have to use `state` instead. Here is an example of
        /// how you can access the database:
        /// ```
        /// let mut nav = MockNavigator::new(db); // db is moved into `new`.
        /// let epics = nav.state
        ///     .clone()
        ///     .last_written_state
        ///     .borrow()
        ///     .epics;
        /// ```
        #[allow(dead_code)]
        // `new` is used for testing. If `warn(dead_code)` is enabled, then cargo check will incorrectly
        // report unused code.
        pub fn new(db: Rc<JiraDatabase>) -> Self {
            Self {
                pages: vec![Box::new(HomePage { db: db.clone() })],
                prompts: Prompt::new(),
                db: db.clone(),
                state: Rc::new(MockDatabase {
                    last_written_state: RefCell::new(db.read().unwrap()),
                }),
            }
        }

        /// `set_prompts` assigns `prompt` to the `MockNavigator`.
        #[allow(dead_code)]
        // `set_prompts` is used for testing. If `warn(dead_code)` is enabled, then cargo check will incorrectly
        // report unused code.
        pub fn set_prompts(&mut self, prompt: Prompt) {
            self.prompts = prompt;
        }
    }

    impl NavigationManager for MockNavigator {
        fn current_page(&self) -> Option<&dyn Page> {
            self.pages.last().map(|page| page.as_ref())
        }

        fn dispatch_action(&mut self, action: Action) -> anyhow::Result<()> {
            match action {
                Action::NavigateToEpicDetail { epic_id } => {
                    let page = Box::new(EpicDetail {
                        epic_id,
                        db: self.db.clone(),
                    });
                    self.pages.push(page);
                }
                Action::NavigateToStoryDetail { story_id, epic_id } => {
                    let page = Box::new(StoryDetail {
                        story_id,
                        epic_id,
                        db: self.db.clone(),
                    });
                    self.pages.push(page);
                }
                Action::NavigateToPreviousPage => {
                    self.pages.pop();
                }
                Action::CreateEpic => {
                    let epic = (self.prompts.create_epic)();
                    self.db.create_epic(&epic)?;
                    self.state = Rc::new(MockDatabase {
                        last_written_state: RefCell::new(self.db.read()?),
                    });
                }
                Action::CreateStory { epic_id } => {
                    let story = (self.prompts.create_story)();
                    self.db.create_story(&story, epic_id)?;
                    self.state = Rc::new(MockDatabase {
                        last_written_state: RefCell::new(self.db.read()?),
                    });
                }
                Action::UpdateEpicName { epic_id } => {
                    let name = (self.prompts.update_name)();
                    self.db.update_epic_name(epic_id, &name)?;
                    self.state = Rc::new(MockDatabase {
                        last_written_state: RefCell::new(self.db.read()?),
                    });
                }
                Action::UpdateEpicDescription { epic_id } => {
                    let description = (self.prompts.update_description)();
                    self.db.update_epic_description(epic_id, &description)?;
                    self.state = Rc::new(MockDatabase {
                        last_written_state: RefCell::new(self.db.read()?),
                    });
                }
                Action::UpdateEpicStatus { epic_id } => {
                    if let Some(status) = (self.prompts.update_status)() {
                        self.db.update_epic_status(epic_id, status)?;
                        self.state = Rc::new(MockDatabase {
                            last_written_state: RefCell::new(self.db.read()?),
                        });
                    }
                }
                Action::UpdateStoryName { story_id } => {
                    let name = (self.prompts.update_name)();
                    self.db.update_story_name(story_id, &name)?;
                    self.state = Rc::new(MockDatabase {
                        last_written_state: RefCell::new(self.db.read()?),
                    });
                }
                Action::UpdateStoryDescription { story_id } => {
                    let description = (self.prompts.update_description)();
                    self.db.update_story_description(story_id, &description)?;
                    self.state = Rc::new(MockDatabase {
                        last_written_state: RefCell::new(self.db.read()?),
                    });
                }
                Action::UpdateStoryStatus { story_id } => {
                    if let Some(status) = (self.prompts.update_status)() {
                        self.db.update_story_status(story_id, status)?;
                        self.state = Rc::new(MockDatabase {
                            last_written_state: RefCell::new(self.db.read()?),
                        });
                    }
                }
                Action::DeleteEpic { epic_id } => {
                    if (self.prompts.delete_epic)() {
                        self.db.delete_epic(epic_id)?;
                        self.state = Rc::new(MockDatabase {
                            last_written_state: RefCell::new(self.db.read()?),
                        });
                        self.pages.pop();
                    }
                }
                Action::DeleteStory { story_id, epic_id } => {
                    if (self.prompts.delete_story)() {
                        self.db.delete_story(story_id, epic_id)?;
                        self.state = Rc::new(MockDatabase {
                            last_written_state: RefCell::new(self.db.read()?),
                        });
                        self.pages.pop();
                    }
                }
                Action::Exit => self.pages.clear(),
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        db::test_utils::MockDatabase,
        models::{Epic, Status, Story},
    };

    use super::*;

    #[test]
    fn should_start_on_home_page() {
        let db = Rc::new(JiraDatabase {
            db: Box::new(MockDatabase::new()),
        });
        let nav = Navigator::new(db.clone());
        assert!(nav.current_page().is_some());
        assert!(nav
            .current_page()
            .unwrap()
            .as_any()
            .downcast_ref::<HomePage>()
            .is_some())
    }

    #[test]
    fn should_navigate_to_epic_detail() {
        let db = Rc::new(JiraDatabase {
            db: Box::new(MockDatabase::new()),
        });
        let epic_id = db.create_epic(&Epic::new("", "")).unwrap();
        let mut nav = Navigator::new(db.clone());

        let res = nav.dispatch_action(Action::NavigateToEpicDetail { epic_id });
        assert!(res.is_ok());
        assert_eq!(nav.page_count(), 2usize);

        let current_page = nav.current_page().unwrap();
        assert!(current_page.as_any().downcast_ref::<EpicDetail>().is_some());
    }

    #[test]
    fn should_navigate_to_story_detail() {
        let db = Rc::new(JiraDatabase {
            db: Box::new(MockDatabase::new()),
        });
        let epic_id = db.create_epic(&Epic::new("", "")).unwrap();
        let story_id = db.create_story(&Story::new("", ""), epic_id).unwrap();
        let mut nav = Navigator::new(db.clone());

        let res = nav.dispatch_action(Action::NavigateToStoryDetail { story_id, epic_id });
        assert!(res.is_ok());
        assert_eq!(nav.page_count(), 2usize);

        let current_page = nav.current_page().unwrap();
        assert!(current_page
            .as_any()
            .downcast_ref::<StoryDetail>()
            .is_some());
    }

    #[test]
    fn should_navigate_to_previous_page() {
        let db = Rc::new(JiraDatabase {
            db: Box::new(MockDatabase::new()),
        });
        let mut nav = Navigator::new(db.clone());

        let epic_id = db.create_epic(&Epic::new("", "")).unwrap();
        nav.dispatch_action(Action::NavigateToEpicDetail { epic_id })
            .unwrap();

        let res = nav.dispatch_action(Action::NavigateToPreviousPage);
        assert!(res.is_ok());
        assert!(nav.current_page().is_some());

        let current_page = nav.current_page().unwrap();
        assert!(current_page.as_any().downcast_ref::<HomePage>().is_some());
    }

    #[test]
    fn should_create_epic() {
        let db = Rc::new(JiraDatabase {
            db: Box::new(MockDatabase::new()),
        });
        let mut prompts = Prompt::new();
        prompts.create_epic = Box::new(|| Epic::new("name", "description"));
        let mut nav = Navigator::new(db.clone());
        nav.set_prompts(prompts);

        let res = nav.dispatch_action(Action::CreateEpic);
        assert!(res.is_ok());

        let epics = &db.read().unwrap().epics;
        assert!(!epics.is_empty());

        let (_, epic) = epics.iter().next().unwrap();
        assert_eq!(epic.name, "name");
        assert_eq!(epic.description, "description");
    }

    #[test]
    fn should_create_story() {
        let db = Rc::new(JiraDatabase {
            db: Box::new(MockDatabase::new()),
        });
        let epic_id = db.create_epic(&Epic::new("name", "description")).unwrap();
        let mut prompts = Prompt::new();
        prompts.create_story = Box::new(|| Story::new("name", "description"));
        let mut nav = Navigator::new(db.clone());
        nav.set_prompts(prompts);

        let res = nav.dispatch_action(Action::CreateStory { epic_id });
        assert!(res.is_ok());

        let stories = &db.read().unwrap().stories;
        assert!(!stories.is_empty());

        let (_, story) = stories.iter().next().unwrap();
        assert_eq!(story.name, "name");
        assert_eq!(story.description, "description");
    }

    #[test]
    fn should_update_epic_status() {
        let db = Rc::new(JiraDatabase {
            db: Box::new(MockDatabase::new()),
        });
        let epic_id = db.create_epic(&Epic::new("name", "description")).unwrap();
        let mut prompts = Prompt::new();
        prompts.update_status = Box::new(|| Some(Status::InProgress));
        let mut nav = Navigator::new(db.clone());
        nav.set_prompts(prompts);

        let res = nav.dispatch_action(Action::UpdateEpicStatus { epic_id });
        assert!(res.is_ok());

        let state = db.read().unwrap();
        let epic = state.epics.get(&epic_id);
        assert!(epic.is_some());
        assert_eq!(epic.unwrap().status, Status::InProgress);
    }

    #[test]
    fn should_update_story_status() {
        let db = Rc::new(JiraDatabase {
            db: Box::new(MockDatabase::new()),
        });
        let epic_id = db.create_epic(&Epic::new("name", "description")).unwrap();
        let story_id = db
            .create_story(&Story::new("name", "description"), epic_id)
            .unwrap();
        let mut prompts = Prompt::new();
        prompts.update_status = Box::new(|| Some(Status::InProgress));
        let mut nav = Navigator::new(db.clone());
        nav.set_prompts(prompts);

        let res = nav.dispatch_action(Action::UpdateStoryStatus { story_id });
        assert!(res.is_ok());

        let state = db.read().unwrap();
        let story = state.stories.get(&story_id);
        assert!(story.is_some());
        assert_eq!(story.unwrap().status, Status::InProgress);
    }

    #[test]
    fn should_delete_epic() {
        let db = Rc::new(JiraDatabase {
            db: Box::new(MockDatabase::new()),
        });
        let epic_id = db.create_epic(&Epic::new("name", "description")).unwrap();
        let mut prompts = Prompt::new();
        prompts.delete_epic = Box::new(|| true);
        let mut nav = Navigator::new(db.clone());
        nav.set_prompts(prompts);

        let res = nav.dispatch_action(Action::DeleteEpic { epic_id });
        assert!(res.is_ok());

        let state = db.read().unwrap();
        assert!(state.epics.is_empty());
    }

    #[test]
    fn should_delete_story() {
        let db = Rc::new(JiraDatabase {
            db: Box::new(MockDatabase::new()),
        });
        let epic_id = db.create_epic(&Epic::new("name", "description")).unwrap();
        let story_id = db
            .create_story(&Story::new("name", "description"), epic_id)
            .unwrap();
        let mut prompts = Prompt::new();
        prompts.delete_story = Box::new(|| true);
        let mut nav = Navigator::new(db.clone());
        nav.set_prompts(prompts);

        let res = nav.dispatch_action(Action::DeleteStory { story_id, epic_id });
        assert!(res.is_ok());

        let state = db.read().unwrap();
        assert!(state.stories.is_empty());
    }

    #[test]
    fn should_exit() {
        let db = Rc::new(JiraDatabase {
            db: Box::new(MockDatabase::new()),
        });
        let mut nav = Navigator::new(db.clone());

        let epic_id = db.create_epic(&Epic::new("", "")).unwrap();
        nav.pages.push(Box::new(EpicDetail {
            epic_id,
            db: db.clone(),
        }));

        let res = nav.dispatch_action(Action::Exit);
        assert!(res.is_ok());
        assert!(nav.pages.is_empty());
    }
}
