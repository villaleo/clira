use crate::{
    models::{Epic, Status, Story, Task},
    ui::pages::MAX_NAME_LENGTH,
    utils::read_line,
};

/// `Prompt` has different members to display prompts and read user input.
/// It acts as a level of indirection for testability.
pub struct Prompt {
    pub create_epic: Box<dyn Fn() -> Option<Epic>>,
    pub create_story: Box<dyn Fn() -> Option<Story>>,
    pub create_task: Box<dyn Fn() -> Option<Task>>,
    pub delete_epic: Box<dyn Fn() -> bool>,
    pub delete_story: Box<dyn Fn() -> bool>,
    pub delete_task: Box<dyn Fn() -> bool>,
    pub update_name: Box<dyn Fn() -> String>,
    pub update_description: Box<dyn Fn() -> String>,
    pub update_status: Box<dyn Fn() -> Option<Status>>,
}

impl Prompt {
    /// `new` creates a new instance of `Prompt` with members ready to use.
    pub fn new() -> Self {
        Self {
            create_epic: Box::new(create_epic),
            create_story: Box::new(create_story),
            create_task: Box::new(create_task),
            delete_epic: Box::new(delete_epic),
            delete_story: Box::new(delete_story),
            delete_task: Box::new(delete_task),
            update_name: Box::new(update_name),
            update_description: Box::new(update_description),
            update_status: Box::new(update_status),
        }
    }
}

fn create_epic() -> Option<Epic> {
    println!("Enter Epic name: ((x) cancel and discard)");
    let name: String = loop {
        match read_line() {
            Some(name) => {
                if name.to_lowercase() == "x" {
                    return None;
                }
                if name.len() >= MAX_NAME_LENGTH {
                    println!(
                        "Epic names should be short and meaningful. Please provide a shorter name:"
                    );
                } else {
                    break name;
                }
            }
            None => continue,
        }
    };
    println!("Enter Epic description: ((x) cancel and discard)");
    let description: String = loop {
        match read_line() {
            Some(description) => {
                if description.to_lowercase() == "x" {
                    return None;
                }
                break description;
            }
            None => continue,
        }
    };
    Some(Epic::new(&name, &description))
}

fn create_story() -> Option<Story> {
    println!("Enter Story name: ((x) cancel and discard)");
    let name: String = loop {
        match read_line() {
            Some(name) => {
                if name.to_lowercase() == "x" {
                    return None;
                }
                if name.len() >= MAX_NAME_LENGTH {
                    println!(
                        "Story names should be short and meaningful. Please provide a shorter name:"
                    );
                } else {
                    break name;
                }
            }
            None => continue,
        }
    };
    println!("Enter Story description: ((x) cancel and discard)");
    let description: String = loop {
        match read_line() {
            Some(description) => {
                if description.to_lowercase() == "x" {
                    return None;
                }
                break description;
            }
            None => continue,
        }
    };
    Some(Story::new(&name, &description))
}

fn create_task() -> Option<Task> {
    println!("Enter Task name: ((x) cancel and discard)");
    let name: String = loop {
        match read_line() {
            Some(name) => {
                if name.to_lowercase() == "x" {
                    return None;
                }
                if name.len() >= MAX_NAME_LENGTH {
                    println!(
                        "Task names should be short and meaningful. Please provide a shorter name:"
                    );
                } else {
                    break name;
                }
            }
            None => continue,
        }
    };
    println!("Enter Task description: ((x) cancel and discard)");
    let description: String = loop {
        match read_line() {
            Some(description) => {
                if description.to_lowercase() == "x" {
                    return None;
                }
                break description;
            }
            None => continue,
        }
    };
    Some(Task::new(&name, &description))
}

fn delete_epic() -> bool {
    println!("Delete this Epic? All Stories in this Epic will also be deleted.");
    println!("\t(y) yes | (n) no");
    read_line()
        .unwrap_or("".into())
        .to_ascii_lowercase()
        .contains('y')
}

fn delete_story() -> bool {
    println!("Delete this Story? All Tasks in this Story will also be deleted.");
    println!("\t(y) yes | (n) no");
    read_line()
        .unwrap_or("".into())
        .to_ascii_lowercase()
        .contains('y')
}

fn delete_task() -> bool {
    println!("Delete this Task?");
    println!("\t(y) yes | (n) no");
    read_line()
        .unwrap_or("".into())
        .to_ascii_lowercase()
        .contains('y')
}

fn update_name() -> String {
    println!("New name:");
    loop {
        match read_line() {
            Some(name) => {
                if name.len() >= MAX_NAME_LENGTH {
                    println!(
                        "Story names should be short and meaningful. Please provide a shorter name:"
                    );
                } else {
                    break name;
                }
            }
            None => continue,
        }
    }
}

fn update_description() -> String {
    println!("New description:");
    read_line().unwrap_or("".into())
}

fn update_status() -> Option<Status> {
    println!("New status:");
    println!("\t(1) Open\n\t(2) In Progress\n\t(3) Resolved\n\t(4) Closed");
    println!("(x) cancel");
    match read_line().unwrap_or("".into()).as_str() {
        "1" => Some(Status::Open),
        "2" => Some(Status::InProgress),
        "3" => Some(Status::Resolved),
        "4" => Some(Status::Closed),
        _ => None,
    }
}
