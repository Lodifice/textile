use crate::char::Character::*;
use crate::char::*;

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
        cat_string!(Cat11, 'a', 'b', 'b', 'a', Cat12, '1', '@', Cat11, 'a', Cat06, '#'),
        result.1
    )
}

#[test]
fn long_category_string() {
    let state = State::default();
    let result = categorize_string(TextileInput::new("this is some \\test and \\_stuff", state))
        .expect("parser error!");
    assert_eq!(
        cat_string!(
            Cat11, 't', 'h', 'i', 's', Cat10, ' ', Cat11, 'i', 's', Cat10, ' ', Cat11, 's', 'o',
            'm', 'e', Cat10, ' ', Cat00, '\\', Cat11, 't', 'e', 's', 't', Cat10, ' ', Cat11, 'a',
            'n', 'd', Cat10, ' ', Cat00, '\\', Cat07, '_', Cat11, 's', 't', 'u', 'f', 'f'
        ),
        result.1
    );
}
