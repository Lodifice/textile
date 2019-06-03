use crate::char::Category::*;
use crate::char::*;

macro_rules! cat_string {
    ( $($c:ident, $($x:literal),*),* ) => {
        vec![$( $(($c, $x)),* ),*]
    }
}

#[cfg(test)]
mod tokenizer_test {
    use crate::char::Category::*;
    use crate::token::Token::*;
    use crate::token::*;

    fn token_vec(input: &str) -> Vec<Token> {
        let tokenizer = Tokenizer::new(input);
        tokenizer.collect()
    }

    #[test]
    fn test_letter() {
        assert_eq!(token_vec("a"), vec![Character('a', Cat11)]);
    }

    #[test]
    fn test_superscript_escape_single() {
        assert_eq!(token_vec("\\^^@"), vec![ControlSequence("\0".into())]);
        assert_eq!(token_vec("\\^^?"), vec![ControlSequence("\u{7f}".into())]);
        assert_eq!(token_vec("\\^^f1"), vec![ControlSequence("\u{f1}".into())]);
        assert_eq!(
            token_vec("\\^^61bc^^V"),
            vec![ControlSequence("abc".into()).into(), Character(' ', Cat10)]
        );
        assert_eq!(
            token_vec("\\^^5ca"),
            vec![ControlSequence("\\".into()), Character('a', Cat11)]
        );
    }
}

#[test]
fn catergory_one_char() {
    let state = State::default();
    let result = categorize_string(TextileInput::new("a", state)).expect("parser error!");
    assert_eq!(vec![(Cat11, 'a')], result.1)
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
