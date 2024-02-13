use std::rc::Rc;

use db::JiraDatabase;
use ui::navigator::{NavigationManager, Navigator};
use utils::read_line;

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
        if nav.current_page().is_none() {
            break;
        }
        let page = nav.current_page().unwrap();
        if let Err(error) = page.draw() {
            println!("Error rendering page: {}", error);
            println!("Press (enter) to continue..");
            let _ = read_line();
        }
        if let Some(line) = read_line() {
            match page.action_from(&line) {
                Ok(action) => {
                    if let Some(action) = action {
                        if let Err(error) = nav.dispatch_action(action) {
                            println!("Error processing request: {}", error);
                            println!("Press (enter) to continue..");
                            let _ = read_line();
                        }
                    }
                }
                Err(error) => {
                    println!("Error reading input: {}", error);
                    println!("Press (enter) to continue..");
                    let _ = read_line();
                }
            }
        }
    }
}
