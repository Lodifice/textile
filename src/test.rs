use crate::Character::*;
use crate::*;

macro_rules! cat_string {
    ( $($c:ident, $($x:literal),*),* ) => {
        vec![$( $($c($x)),* ),*]
    }
}

#[test]
fn catergory_one_char() {
    let state = State::default();
    let result = categorize_string(TextileInput::new("a", state)).expect("parser error!");
    assert_eq!(vec![Cat11('a')], result.1)
}

#[test]
fn catergory_string() {
    let state = State::default();
    let result = categorize_string(TextileInput::new("abba1@a#", state)).expect("parser error!");
    assert_eq!(
        cat_string!(Cat11, 'a', 'b', 'b', 'a', Cat12, '1', '@', 'a'),
        result.1
    )
}
