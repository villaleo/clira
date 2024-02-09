use std::rc::Rc;

use db::JiraDatabase;
use ui::navigator::{NavigationManager, Navigator};

use crate::utils::read_input;

mod db;
mod models;
mod ui;
mod utils;

fn main() {
    let file_path = "data/db.json";
    let db =
        Rc::new(JiraDatabase::new(file_path).expect("failed to load database file into program"));
    let mut nav = Navigator::new(db.clone());

    loop {
        clearscreen::clear().expect("failed to clear the screen");

        if let Some(page) = nav.current_page() {
            if let Err(error) = page.draw() {
                println!("Error rendering page: {}", error);
                println!("Press (enter) to continue..");
                let _ = read_input();
            }

            match page.action_from(&read_input()) {
                Err(error) => {
                    println!("Error reading input: {}", error);
                    println!("Press (enter) to continue..");
                    let _ = read_input();
                }
                Ok(action) => {
                    if let Some(action) = action {
                        if let Err(error) = nav.dispatch_action(action) {
                            println!("Error processing request: {}", error);
                            println!("Press (enter) to continue..");
                            let _ = read_input();
                        }
                    }
                }
            }
        } else {
            break;
        }
    }
}
