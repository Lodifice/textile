#[cfg(test)]
mod tokenizer_test {
    use crate::token::{Category::*, Token::*, *};

    fn token_vec(input: &str) -> Vec<Token> {
        let tokenizer = Tokenizer::new(input.lines().map(|s| s.to_owned()));
        tokenizer.collect()
    }

    #[test]
    fn test_letter() {
        assert_eq!(
            token_vec("a"),
            vec![Character('a', Cat11), Character(' ', Cat10)]
        );
    }

    #[test]
    fn test_spaces() {
        assert_eq!(
            token_vec("a b"),
            vec![
                Character('a', Cat11),
                Character(' ', Cat10),
                Character('b', Cat11),
                Character(' ', Cat10)
            ]
        );
        assert_eq!(
            token_vec("a    b"),
            vec![
                Character('a', Cat11),
                Character(' ', Cat10),
                Character('b', Cat11),
                Character(' ', Cat10)
            ]
        );
        assert_eq!(token_vec("\\test  "), vec![ControlSequence("test".into())]);
        assert_eq!(
            token_vec("\\test\t  b"),
            vec![
                ControlSequence("test".into()),
                Character('b', Cat11),
                Character(' ', Cat10)
            ]
        );
    }

    #[test]
    fn test_lines() {
        assert_eq!(
            token_vec("ab  "),
            vec![
                Character('a', Cat11),
                Character('b', Cat11),
                Character(' ', Cat10)
            ]
        );
        assert_eq!(
            token_vec("ab  \n"),
            vec![
                Character('a', Cat11),
                Character('b', Cat11),
                Character(' ', Cat10)
            ]
        );
        assert_eq!(
            token_vec("ab  \n\na"),
            vec![
                Character('a', Cat11),
                Character('b', Cat11),
                Character(' ', Cat10),
                ControlSequence("par".into()),
                Character('a', Cat11),
                Character(' ', Cat10)
            ]
        );
        assert_eq!(
            token_vec("ab  \n\n \n \\a"),
            vec![
                Character('a', Cat11),
                Character('b', Cat11),
                Character(' ', Cat10),
                ControlSequence("par".into()),
                ControlSequence("par".into()),
                ControlSequence("a".into()),
            ]
        );
        assert_eq!(
            token_vec("ab^^Mdefgh  \n\\a"),
            vec![
                Character('a', Cat11),
                Character('b', Cat11),
                Character(' ', Cat10),
                ControlSequence("a".into()),
            ]
        );
    }

    #[test]
    fn test_superscript_escape_single() {
        assert_eq!(
            token_vec("\\^^@"),
            vec![ControlSequence("\0".into()), Character(' ', Cat10)]
        );
        assert_eq!(
            token_vec("\\^^?"),
            vec![ControlSequence("\u{7f}".into()), Character(' ', Cat10)]
        );
        assert_eq!(
            token_vec("\\^^f1"),
            vec![ControlSequence("\u{f1}".into()), Character(' ', Cat10)]
        );
        assert_eq!(
            token_vec("\\^^61bc~ a"),
            vec![
                ControlSequence("abc".into()).into(),
                Character('~', Cat13),
                Character(' ', Cat10),
                Character('a', Cat11),
                Character(' ', Cat10)
            ]
        );
        assert_eq!(
            token_vec("\\^^61bc        "),
            vec![ControlSequence("abc".into())]
        );
        assert_eq!(
            token_vec("\\^^5ca"),
            vec![
                ControlSequence("\\".into()),
                Character('a', Cat11),
                Character(' ', Cat10)
            ]
        );
    }
}
