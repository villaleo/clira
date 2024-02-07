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

#[cfg(test)]
mod tests {
    use super::constrain_text;

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
}
