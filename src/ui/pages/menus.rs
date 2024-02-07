use tabled::{
    builder::Builder,
    settings::{self, object::Segment, Alignment},
};

use super::{EpicDetail, HomePage, Page, StoryDetail};

/// `Menu` is a trait with a single method `draw_menu`. It requires that Page
/// be implemented as well.
pub trait Menu: Page {
    /// `draw_menu` draws a menu to the page with all the actions available
    /// to the user.
    fn draw_menu(&self);
}

impl Menu for HomePage {
    fn draw_menu(&self) {
        let menu = into_table(&["(q) quit", "(n) new epic", "<ID> view epic"]);
        println!("\n{}", menu);
    }
}

impl Menu for EpicDetail {
    fn draw_menu(&self) {
        let menu = into_table(&[
            "(b) back",
            "(u) update",
            "(d) delete",
            "(n) new story",
            "<ID> view story",
        ]);
        println!("\n{}", menu);
    }
}

impl Menu for StoryDetail {
    fn draw_menu(&self) {
        let menu = into_table(&["(b) back", "(u) update", "(d) delete"]);
        println!("\n{}", menu);
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
