#[cfg(test)]
mod tokenizer_test {
    use crate::token::{Category::*, OtherToken::*, Token::*, *};

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
                Other(Skipped("   ".into()), Span::new(1, 2, 4)),
                Character('b', Cat11),
                Character(' ', Cat10)
            ]
        );
        assert_eq!(
            // end-of-line spaces are deleted upon read
            token_vec("\\test  "),
            vec![ControlSequence("test".into(), Span::new(1, 0, 4)),]
        );
        assert_eq!(
            token_vec("\\test\t  b"),
            vec![
                ControlSequence("test".into(), Span::new(1, 0, 4)),
                Other(Skipped("\t  ".into()), Span::new(1, 5, 7)),
                Character('b', Cat11),
                Character(' ', Cat10)
            ]
        );
        assert_eq!(
            token_vec("\\\\  b"),
            vec![
                ControlSequence("\\".into(), Span::new(1, 0, 1)),
                // first space is preserved because of non-letter CS
                Character(' ', Cat10),
                Other(Skipped(" ".into()), Span::new(1, 3, 3)),
                Character('b', Cat11),
                Character(' ', Cat10)
            ]
        );
        assert_eq!(
            token_vec("\\ a"),
            vec![
                ControlSequence(" ".into(), Span::new(1, 0, 1)),
                Character('a', Cat11),
                Character(' ', Cat10),
            ]
        );
        assert_eq!(
            token_vec("\\test\t  %  abc"),
            vec![
                ControlSequence("test".into(), Span::new(1, 0, 4)),
                Other(Skipped("\t  ".into()), Span::new(1, 5, 7)),
                Other(Comment("  abc\r".into()), Span::new(1, 8, 14))
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
                ControlSequence("par".into(), Span::any()),
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
                ControlSequence("par".into(), Span::any()),
                // here, the space is ignored, because of line preprocessing
                ControlSequence("par".into(), Span::any()),
                Other(Skipped(" ".into()), Span::new(4, 0, 0)),
                ControlSequence("a".into(), Span::new(4, 1, 2)),
            ]
        );
        assert_eq!(
            token_vec("ab^^Mdefgh  \n\\a"),
            vec![
                Character('a', Cat11),
                Character('b', Cat11),
                Other(Skipped("defgh\r".into()), Span::new(1, 2, 10)),
                Character(' ', Cat10),
                ControlSequence("a".into(), Span::new(2, 0, 1)),
            ]
        );
    }

    #[test]
    fn test_superscript_escape_single() {
        assert_eq!(
            token_vec("\\^^@"),
            vec![
                ControlSequence("\0".into(), Span::new(1, 0, 3)),
                Character(' ', Cat10)
            ]
        );
        assert_eq!(
            token_vec("\\^^?"),
            vec![
                ControlSequence("\u{7f}".into(), Span::new(1, 0, 3)),
                Character(' ', Cat10)
            ]
        );
        assert_eq!(
            token_vec("\\^^f1"),
            vec![
                ControlSequence("\u{f1}".into(), Span::new(1, 0, 4)),
                Character(' ', Cat10)
            ]
        );
        assert_eq!(
            token_vec("\\^^61bc~ a"),
            vec![
                ControlSequence("abc".into(), Span::new(1, 0, 6)),
                Character('~', Cat13),
                Character(' ', Cat10),
                Character('a', Cat11),
                Character(' ', Cat10)
            ]
        );
        assert_eq!(
            token_vec("\\^^61bc        "),
            vec![ControlSequence("abc".into(), Span::new(1, 0, 6))]
        );
        assert_eq!(
            token_vec("\\^^5ca"),
            vec![
                ControlSequence("\\".into(), Span::new(1, 0, 4)),
                Character('a', Cat11),
                Character(' ', Cat10)
            ]
        );
        assert_eq!(
            token_vec("\\^-A"),
            vec![
                ControlSequence("^".into(), Span::new(1, 0, 1)),
                Character('-', Cat12),
                Character('A', Cat11),
                Character(' ', Cat10)
            ]
        );
    }

    #[test]
    fn test_hidden_categories() {
        assert_eq!(
            token_vec("he\0llo"),
            vec![
                Character('h', Cat11),
                Character('e', Cat11),
                Other(IgnoredCharacter('\0'), Span::new(1, 2, 2)),
                Character('l', Cat11),
                Character('l', Cat11),
                Character('o', Cat11),
                Character(' ', Cat10)
            ]
        );
        assert_eq!(
            token_vec("he\0llo\x01\x1f"),
            vec![
                Character('h', Cat11),
                Character('e', Cat11),
                Other(IgnoredCharacter('\0'), Span::new(1, 2, 2)),
                Character('l', Cat11),
                Character('l', Cat11),
                Character('o', Cat11),
                Other(InvalidCharacter('\x01'), Span::new(1, 6, 6)),
                Other(InvalidCharacter('\x1f'), Span::new(1, 7, 7)),
                Character(' ', Cat10)
            ]
        );
    }

    #[test]
    fn change_catcode() {
        fn tokenize(
            input: &'static str,
            mapping: &Fn(Box<&mut dyn TokenizerInteraction>, &Token),
        ) -> Vec<Token> {
            let mut result: Vec<Token> = vec![];
            let mut tokenizer = Tokenizer::new(input.lines().map(|s| s.to_owned()));
            loop {
                let token = match tokenizer.next() {
                    Some(t) => t,
                    None => break,
                };
                mapping(Box::new(&mut tokenizer), &token);
                result.push(token);
            }
            result
        };

        assert_eq!(
            tokenize("a \\a b", &|t, token| {
                if let ControlSequence(_, _) = token {
                    t.catcode(' ', Cat13)
                }
            }),
            vec![
                Character('a', Cat11),
                Character(' ', Cat10),
                ControlSequence("a".into(), Span::new(1, 2, 3)),
                Character(' ', Cat13),
                Character('b', Cat11),
                Character(' ', Cat13),
            ]
        );

        assert_eq!(
            tokenize("abcdef hello world\n\\a~hhh", &|t, token| {
                match token {
                    Character(' ', _) => t.catcode('h', Cat5),
                    Character('~', _) => t.catcode('h', Cat11),
                    _ => (),
                }
            }),
            vec![
                Character('a', Cat11),
                Character('b', Cat11),
                Character('c', Cat11),
                Character('d', Cat11),
                Character('e', Cat11),
                Character('f', Cat11),
                Character(' ', Cat10),
                Other(
                    OtherToken::Skipped("ello world\r".into()),
                    Span::new(1, 7, 18)
                ),
                // no additional space here, as the tokenizer was in SkippingBlanks before
                // the line ending
                ControlSequence("a".into(), Span::new(2, 0, 1)),
                Character('~', Cat13),
                Character('h', Cat11),
                Character('h', Cat11),
                Character('h', Cat11),
                Character(' ', Cat10),
            ]
        );
    }
}
