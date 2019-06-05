use crate::interval_map::{IntIntervalMap, IntervalMap};
use std::char::from_u32;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Category {
    Cat00, // Escape character (/)
    Cat01, // Begin group ({)
    Cat02, // End group (})
    Cat03, // Math shift ($)
    Cat04, // Alignment (&)
    Cat05, // End of line
    Cat06, // Macro parameter (#)
    Cat07, // Math superscript (_)
    Cat08, // Math subscript (^)
    Cat09, // Ignored
    Cat10, // Space
    Cat11, // Letter
    Cat12, // Other (numbers, special characters)
    Cat13, // Active character (~)
    Cat14, // Start of comment (%)
    Cat15, // Invalid input ([DEL])
}

use Category::*;

/// Tokens not normally produced by TeX
#[derive(Debug, PartialEq, Clone)]
pub enum OtherToken {
    Comment(String),
    /// A character of class 9
    IgnoredCharacter(char),
    /// A character of class 15
    InvalidCharacter(char),
    /// Input which was skipped, e.g. by a premature end of line
    /// or by skipping spaces.
    Skipped(String),
    /// A character which was represented by an escape sequence
    EscapedCharacter(String),
}

/// A location in the input file.
#[derive(Debug, Clone)]
pub struct Span {
    /// Line *number* the current token is generated from
    pub line: usize,
    /// Index of the first column of the span
    pub start: usize,
    /// Index of the last column of the span
    pub end: usize,
}

impl Span {
    pub fn new(line: usize, start: usize, end: usize) -> Self {
        Span { line, start, end }
    }

    pub fn extend(&mut self, step: usize) {
        self.end += step;
    }

    /// Dummy span, which is equal to any other span.
    pub fn any() -> Self {
        Span {
            line: 0,
            start: 0,
            end: 0,
        }
    }
}

impl PartialEq for Span {
    fn eq(&self, other: &Self) -> bool {
        self.start == other.start && self.end == other.end && self.line == other.line
            || self.start == 0 && self.end == 0 && self.line == 0
            || other.start == 0 && other.end == 0 && other.line == 0
    }
}

/// Tokens as described in chapter 7 of the texbook
#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    /// A TeX control sequence.
    ControlSequence(String, Span),
    /// A single TeX character with its category.
    Character(char, Category),
    /// A non-TeX token, useful for diagnostics
    Other(OtherToken, Span),
}

/// The tokenizer states as described in chapter 8 of the texbook
#[derive(Debug, PartialEq, Clone)]
pub enum TokenizerState {
    LineStart,
    LineMiddle,
    SkippingBlanks,
}

#[derive(Debug)]
pub struct Tokenizer<L> {
    category_map: IntIntervalMap<u32, Category>,
    state: TokenizerState,
    lines: L,
    /// Buffer holding the current line
    line: String,
    line_count: usize,

    /// Pointer to the current buffer position
    pos: usize,
    endlinechar: char,

    /// Buffer of tokens. Alwas emptied before more TeX tokens are generated.
    token_buffer: Vec<Token>,
}

impl<L: Iterator<Item = String>> Iterator for Tokenizer<L> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        // return sytax tokens first, if available
        if let Some(t) = self.token_buffer.pop() {
            return Some(t);
        };

        let mut here = self.here();
        let chr;
        loop {
            match self.pop_char() {
                Some(c) => {
                    chr = c;
                    break;
                }
                // try to read next line
                None => {
                    if !self.next_line() {
                        return None;
                    }
                    self.state = TokenizerState::LineStart;
                    here = self.here();
                }
            };
        }
        here.end = self.pos - 1;

        let cat = self.cat(chr);

        // process state as described in chapter 8, p. 46 of the texbook
        match cat {
            Cat00 => match self.pop_char() {
                None => self.push(Token::ControlSequence(String::new(), here)),
                Some(c) => {
                    let mut content = String::new();
                    content.push(c);
                    match self.cat(c) {
                        Cat11 => {
                            loop {
                                match self.look_ahead() {
                                    Some(c) if self.cat(c) == Cat11 => {
                                        self.pop_char();
                                        content.push(c);
                                    }
                                    _ => break,
                                }
                            }
                            self.state = TokenizerState::SkippingBlanks;
                        }
                        Cat10 => self.state = TokenizerState::SkippingBlanks,
                        _ => self.state = TokenizerState::LineMiddle,
                    };
                    here.end = self.pos - 1;
                    self.push(Token::ControlSequence(content, here));
                }
            },
            Cat01 | Cat02 | Cat03 | Cat04 | Cat06 | Cat07 | Cat08 | Cat11 | Cat12 | Cat13 => {
                self.state = TokenizerState::LineMiddle;
                self.push(Token::Character(chr, cat))
            }
            Cat05 => {
                // throw away rest of line
                let mut skipped = String::new();
                while let Some(c) = self.pop_char() {
                    skipped.push(c)
                }
                if !skipped.is_empty() {
                    let mut loc = here.clone();
                    loc.end = self.pos - 1;
                    self.push(Token::Other(OtherToken::Skipped(skipped), loc));
                }
                match self.state {
                    TokenizerState::LineStart => {
                        self.push(Token::ControlSequence("par".into(), here))
                    }
                    TokenizerState::LineMiddle => self.push(Token::Character(' ', self.cat(' '))),
                    TokenizerState::SkippingBlanks => (),
                }
            }
            Cat09 => {
                self.push(Token::Other(OtherToken::IgnoredCharacter(chr), here));
            }
            Cat10 => match self.state {
                TokenizerState::LineStart | TokenizerState::SkippingBlanks => {
                    let mut whitespace = String::new();
                    whitespace.push(chr);
                    let mut loc = here.clone();
                    loop {
                        match self.look_ahead() {
                            Some(c) if self.cat(c) == Cat10 => {
                                self.pop_char();
                                whitespace.push(c);
                            }
                            _ => break,
                        }
                    }
                    loc.end = self.pos - 1;
                    self.push(Token::Other(OtherToken::Skipped(whitespace), loc))
                }
                TokenizerState::LineMiddle => {
                    self.state = TokenizerState::SkippingBlanks;
                    self.push(Token::Character(' ', self.cat(' ')))
                }
            },
            Cat14 => {
                eprintln!("comment with  {:?}", chr);
                let mut comment = String::new();
                while let Some(c) = self.pop_char() {
                    comment.push(c);
                }
                here.end = self.pos - 1;
                self.push(Token::Other(OtherToken::Comment(comment), here));
            }
            Cat15 => {
                self.push(Token::Other(OtherToken::InvalidCharacter(chr), here));
            }
        };
        self.next()
    }
}

macro_rules! assign {
    ($map:ident, $lo:literal, $hi:literal, $cls:ident) => {
        #[allow(clippy::range_plus_one)]
        $map.assign(($lo as u32)..($hi as u32 + 1), Category::$cls);
    };
    ($map:ident, $idx:literal, $cls:ident) => {
        $map.assign_single($idx as u32, Category::$cls);
    };
}

impl<L: Iterator<Item = String>> Tokenizer<L> {
    /// Span of the next input character
    fn here(&self) -> Span {
        Span::new(self.line_count, self.pos, self.pos)
    }

    /// The input from the current position
    fn input(&self) -> &str {
        &self.line[self.pos..]
    }

    /// Push a syntax token into the buffer.
    fn push(&mut self, token: Token) {
        self.token_buffer.insert(0, token);
    }

    /// Advance to the next line of input.
    /// Preprocessing is done as described on p. 46 of the texbook.
    ///
    /// Returns if the operation was successful, i.e. returns false
    /// if the end of input was reached.
    #[must_use = "the end of input must be handled"]
    fn next_line(&mut self) -> bool {
        self.state = TokenizerState::LineStart;
        let mut line = match self.lines.next() {
            Some(l) => l,
            None => return false,
        };
        line.truncate(line.trim_end_matches(' ').len());
        line.push(self.endlinechar);
        self.line = line;
        self.pos = 0;
        self.line_count += 1;
        true
    }

    /// pop the next character from the current line.
    /// the character might have been esacped,
    /// which consumes more input than one character.
    fn pop_char(&mut self) -> Option<char> {
        match self.parse_superscript_char() {
            Some((c, l)) => {
                self.pos += l;
                Some(c)
            }
            None => match self.input().chars().next() {
                Some(c) => {
                    self.pos += 1;
                    Some(c)
                }
                None => None,
            },
        }
    }

    /// get the next character of the input
    /// with escaped characters normalized
    fn look_ahead(&self) -> Option<char> {
        match self.parse_superscript_char() {
            Some((c, _)) => Some(c),
            None => self.input().chars().next(),
        }
    }

    fn cat(&self, c: char) -> Category {
        self.category_map.get(c as u32)
    }
    /// Parse a superscript-escaped character (e.g. ^^A or ^^0f).
    ///
    /// Returns the replacement character and length of consumed input, if successful
    fn parse_superscript_char(&self) -> Option<(char, usize)> {
        let mut chars = self.input().chars();
        let c_start = match chars.next().filter(|c| self.cat(*c) == Cat07) {
            Some(c) => c,
            None => return None,
        };
        if chars.next() == Some(c_start) {
            let next_two = [chars.next(), chars.next()];

            let are_hexdigits = next_two.iter().all(|o| {
                o.map(|c| (c.is_ascii_hexdigit() && c.is_lowercase()) || c.is_numeric())
                    .unwrap_or(false)
            });

            if are_hexdigits {
                let chr = from_u32(
                    u32::from_str_radix(&self.input()[2..4], 16)
                        .expect("parse error with superscript-escaped hex character"),
                )
                .expect("unicode error in superscript-escaped character!");
                return Some((chr, 4));
            }

            if let Some(c) = next_two[0] {
                if c as u32 >= 128 {
                    return None;
                }

                let chr = if (c as u32) < 64 {
                    from_u32(c as u32 + 64).unwrap()
                } else {
                    from_u32(c as u32 - 64).unwrap()
                };
                return Some((chr, 3));
            }
        }
        None
    }

    pub fn new(lines: L) -> Self {
        let mut map = IntIntervalMap::new(Category::Cat12);

        assign!(map, '\\', Cat00);
        assign!(map, '{', Cat01);
        assign!(map, '}', Cat02);
        assign!(map, '$', Cat03);
        assign!(map, '&', Cat04);
        assign!(map, '\n', Cat05);
        assign!(map, '\r', Cat05);
        assign!(map, '#', Cat06);
        assign!(map, '^', Cat07);
        assign!(map, '_', Cat08);
        assign!(map, '\0', Cat09);
        assign!(map, ' ', Cat10);
        assign!(map, '\t', Cat10);
        assign!(map, 'a', 'z', Cat11);
        assign!(map, 'A', 'Z', Cat11);
        assign!(map, '0', '9', Cat12);
        assign!(map, '0', '9', Cat12);
        assign!(map, ':', '@', Cat12);

        assign!(map, '~', Cat13);
        assign!(map, '%', Cat14);
        assign!(map, '\x01', '\x08', Cat15);
        assign!(map, '\x0b', Cat15);
        assign!(map, '\x0c', Cat15);
        assign!(map, '\x0e', '\x1f', Cat15);

        Tokenizer {
            category_map: map,
            state: TokenizerState::LineStart,
            lines,
            line: String::new(),
            endlinechar: '\r',
            pos: 0,
            token_buffer: vec![],
            line_count: 0,
        }
    }
}
