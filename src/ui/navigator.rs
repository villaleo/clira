use std::rc::Rc;

use crate::{
    db::JiraDatabase,
    models::Action,
    ui::pages::{prompts::Prompt, EpicDetail, HomePage, Page, StoryDetail},
};

/// `Navigator` manages the navigation stack between different pages.
pub struct Navigator {
    pages: Vec<Box<dyn Page>>,
    prompts: Prompt,
    db: Rc<JiraDatabase>,
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

    /// `current_page` gets the current page that is rendered on the stack.
    #[allow(clippy::borrowed_box)]
    pub fn current_page(&self) -> Option<&Box<dyn Page>> {
        self.pages.last()
    }

    /// `dispatch_action` commits the `action` to the database.
    pub fn dispatch_action(&mut self, action: Action) -> anyhow::Result<()> {
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
                    self.pages.pop();
                }
            }
            Action::Exit => self.pages.clear(),
        }
        Ok(())
    }

    fn page_count(&self) -> usize {
        self.pages.len()
    }

    fn set_prompts(&mut self, prompt: Prompt) {
        self.prompts = prompt;
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

    // #[test]
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
