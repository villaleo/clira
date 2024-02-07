use std::io::stdin;

use crate::models::{Epic, Status, Story};

/// `Prompt` has different members to display prompts and read user input.
/// It acts as a level of indirection for testability.
pub struct Prompt {
    pub create_epic: Box<dyn Fn() -> Epic>,
    pub create_story: Box<dyn Fn() -> Story>,
    pub delete_epic: Box<dyn Fn() -> bool>,
    pub delete_story: Box<dyn Fn() -> bool>,
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
            update_status: Box::new(update_status),
        }
    }
}

fn create_epic() -> Epic {
    println!("Enter name:");
    let mut name = read_input();
    loop {
        if name.is_empty() {
            println!("Name cannot be empty. Please enter a name:");
            name = read_input();
        } else {
            break;
        }
    }
    println!("Enter description:");
    let description = read_input();
    Epic::new(&name, &description)
}

fn create_story() -> Story {
    println!("Enter name:");
    let mut name = read_input();
    loop {
        if name.is_empty() {
            println!("Name cannot be empty. Please enter a name:");
            name = read_input();
        } else {
            break;
        }
    }
    println!("Enter description:");
    let description = read_input();
    Story::new(&name, &description)
}

fn delete_epic() -> bool {
    println!("Delete this epic? All stories in this epic will also be deleted.");
    println!("(y) yes | (n) no");
    let choice = read_input();
    choice.to_ascii_lowercase().contains('y')
}

fn delete_story() -> bool {
    println!("Delete this story?");
    println!("(y) yes | (n) no");
    let choice = read_input();
    choice.to_ascii_lowercase().contains('y')
}

fn update_status() -> Option<Status> {
    println!("Update status:");
    println!("\t1 - Open\n\t2 - In Progress\n\t3 - Resolved\n\t4 - Closed");
    println!("(x) cancel");
    let choice = read_input();
    match choice.as_str() {
        "1" => Some(Status::Open),
        "2" => Some(Status::InProgress),
        "3" => Some(Status::Resolved),
        "4" => Some(Status::Closed),
        _ => None,
    }
}

fn read_input() -> String {
    let mut input = String::new();
    stdin().read_line(&mut input).unwrap();
    input.trim().into()
}
