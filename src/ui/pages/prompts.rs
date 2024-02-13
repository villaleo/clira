use crate::{
    models::{Epic, Status, Story},
    ui::pages::MAX_NAME_LENGTH,
    utils::read_line,
};

/// `Prompt` has different members to display prompts and read user input.
/// It acts as a level of indirection for testability.
pub struct Prompt {
    pub create_epic: Box<dyn Fn() -> Epic>,
    pub create_story: Box<dyn Fn() -> Story>,
    pub delete_epic: Box<dyn Fn() -> bool>,
    pub delete_story: Box<dyn Fn() -> bool>,
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
            delete_epic: Box::new(delete_epic),
            delete_story: Box::new(delete_story),
            update_name: Box::new(update_name),
            update_description: Box::new(update_description),
            update_status: Box::new(update_status),
        }
    }
}

fn create_epic() -> Epic {
    println!("Epic name:");
    let name: String = loop {
        match read_line() {
            Some(name) => {
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
    println!("Epic description:");
    let description: String = loop {
        match read_line() {
            Some(description) => break description,
            None => continue,
        }
    };
    Epic::new(&name, &description)
}

fn create_story() -> Story {
    println!("Story name:");
    let name: String = loop {
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
    };
    println!("Story description:");
    let description: String = loop {
        match read_line() {
            Some(description) => break description,
            None => continue,
        }
    };
    Story::new(&name, &description)
}

fn delete_epic() -> bool {
    println!("Delete this Epic? All Stories in this Epic will also be deleted.");
    println!("(y) yes | (n) no");
    read_line()
        .unwrap_or("".into())
        .to_ascii_lowercase()
        .contains('y')
}

fn delete_story() -> bool {
    println!("Delete this Story?");
    println!("(y) yes | (n) no");
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
