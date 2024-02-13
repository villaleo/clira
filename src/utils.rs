use owo_colors::OwoColorize;
use std::io::stdin;
use tabled::settings::Color;

use crate::models::Status;

/// `read_line` reads an entire line from `stdin` and returns `Some` string with
/// the leading and trailing whitespace removed. `None` is returned if an empty
/// string is read.
pub fn read_line() -> Option<String> {
    let mut input = String::new();
    stdin().read_line(&mut input).unwrap();
    let input = input.trim().to_string();
    if input.is_empty() {
        None
    } else {
        Some(input)
    }
}

/// `constrain_text` breaks a long, single-line string into a multi-line string
/// with smart new-line breaks before a word begins. `line_limit` specifies how
/// long a line needs to be before a new line is inserted.
pub fn constrain_text(text: &str, line_limit: usize) -> String {
    let words: Vec<_> = text.trim().split(' ').collect();
    let mut fmt_text = String::new();
    let mut line_count = 0usize;
    for word in words {
        if line_count >= line_limit || line_count + word.len() >= line_limit {
            fmt_text += &format!("\n{}", word);
            line_count = word.len() + 1;
        } else {
            fmt_text += &format!(" {}", word);
            line_count += word.len() + 1;
        }
    }
    fmt_text.trim().to_owned()
}

/// `color_table_column` parses `status` as the `Status` type, colors it
/// according to its state, and returns it again as a string. Non-status returns
/// the input string.
pub fn color_table_column(status: &str) -> String {
    match Status::from(status.to_owned()) {
        Status::Open => status.to_string(),
        Status::InProgress => status.yellow().to_string(),
        Status::Resolved => status.blue().to_string(),
        Status::Closed => status.green().to_string(),
    }
}

/// `color_for_table_header` returns the `Color` for the `status`, parsed as `Status`.
pub fn color_for_table_header(status: &str) -> Color {
    match Status::from(status.to_owned()) {
        Status::Open => Color::empty(),
        Status::InProgress => Color::FG_YELLOW,
        Status::Resolved => Color::FG_BLUE,
        Status::Closed => Color::FG_GREEN,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constrain_text_should_succeed() {
        let text = "This is a really long, single-line message that will be ".to_owned()
            + "trimmed down into a nice and readable multi-line string.\n";
        assert_eq!(
            constrain_text(&text, 60usize),
            "This is a really long, single-line message that will be\n".to_owned()
                + "trimmed down into a nice and readable multi-line string."
        );
        let text = "This will be very interesting";
        assert_eq!(
            constrain_text(text, 10usize),
            "This will\nbe very\ninteresting"
        )
    }

    #[test]
    fn color_table_column_should_succeed() {
        assert_eq!(color_table_column("foo"), "foo");
        assert_eq!(color_table_column("Open"), "Open");
        assert_eq!(
            color_table_column("In Progress"),
            "In Progress".yellow().to_string()
        );
        assert_eq!(
            color_table_column("Resolved"),
            "Resolved".blue().to_string()
        );
        assert_eq!(color_table_column("Closed"), "Closed".green().to_string());
    }

    #[test]
    fn color_for_table_header_should_succeed() {
        assert_eq!(color_for_table_header("Open"), Color::empty());
        assert_eq!(color_for_table_header("In Progress"), Color::FG_YELLOW);
        assert_eq!(color_for_table_header("Resolved"), Color::FG_BLUE);
        assert_eq!(color_for_table_header("Closed"), Color::FG_GREEN);
    }
}
