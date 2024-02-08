use std::io::stdin;

/// `read_input` reads an entire line from `stdin` and returns a new, trimmed
/// version with the leading and trailing whitespace removed.
pub fn read_input() -> String {
    let mut input = String::new();
    stdin().read_line(&mut input).unwrap();
    input.trim().into()
}
