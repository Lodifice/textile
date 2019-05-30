use tex_parser::*;

fn main() {
    let state = State::default();
    println!(
        "{:?}",
        categorize_character(TextileInput::new("a", state.clone()))
    );
    println!(
        "{:?}",
        categorize_string(TextileInput::new("abba1@a#", state))
    );
}
