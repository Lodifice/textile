use crate::interval_map::{IntIntervalMap, IntervalMap};
use std::char::from_u32;

/// TeX character codes, as defined on p. 37 of the Texbook.
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Category {
    /// Escape character (/)
    Cat0,
    /// Begin group ({)
    Cat1,
    /// End group (})
    Cat2,
    /// Math shift ($)
    Cat3,
    /// Alignment (&)
    Cat4,
    /// End of line
    Cat5,
    /// Macro parameter (#)
    Cat6,
    /// Math superscript (^)
    Cat7,
    /// Math subscript (_)
    Cat8,
    /// Ignored
    Cat9,
    /// Space
    Cat10,
    /// Letter
    Cat11,
    /// Other (numbers, special characters)
    Cat12,
    /// Active character (~)
    Cat13,
    /// Start of comment (%)
    Cat14,
    /// Invalid input ([DEL])
    Cat15,
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
    /// A parameter token (see section 2.7.4 of TeX by Topic)
    ///
    /// This token type is not directly emitted by the tokenizer,
    /// but constructed in macro definitions. This is because parameter
    /// token construction can fail, as only digits and another parameter character
    /// is allowed to follow.
    Parameter(u8),
    /// A non-TeX token, useful for diagnostics
    Other(OtherToken, Span),
}

/// The tokenizer states as described in chapter 8 of the texbook
#[derive(Debug, PartialEq, Clone)]
enum TokenizerState {
    LineStart,
    LineMiddle,
    SkippingBlanks,
}

/// A token generator for TeX.
///
/// Takes an iterator over input lines and transforms it to a sequence
/// of tokens. The behaviour of the tokenizer can be changed mid-way changing
/// the category code of a character (see TeXbook p. 37).
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

/// Defines how the tokenizer may be interacted with during tokenization.
pub trait TokenizerInteraction {
    /// Change the category of character `chr` to `cat`.
    ///
    /// This changes the behaviour of the tokenizer for subsequent tokens.
    /// For more information, refer to page 39 of the TeXbook.
    fn catcode(&mut self, chr: char, category: Category);

    /// Change the endlinechar to `chr`. (See p. 48 of the TeXBook).
    ///
    /// If greater than 255, no character is appended to the line,
    /// which is equivalent to ending the line with a comment in plain TeX.
    fn set_endlinechar(&mut self, chr: char);

    /// Get the current value of \endlinechar.
    fn get_endlinechar(&self) -> char;
}

impl<L: Iterator<Item = String>> Iterator for Tokenizer<L> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        // emtpy token buffer first, if available
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
            Cat0 => match self.pop_char() {
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
            Cat1 | Cat2 | Cat3 | Cat4 | Cat6 | Cat7 | Cat8 | Cat11 | Cat12 | Cat13 => {
                self.state = TokenizerState::LineMiddle;
                self.push(Token::Character(chr, cat))
            }
            Cat5 => {
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
            Cat9 => {
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

impl<L: Iterator<Item = String>> TokenizerInteraction for Tokenizer<L> {
    fn catcode(&mut self, chr: char, cat: Category) {
        self.category_map.assign_single(chr as u32, cat);
    }

    fn set_endlinechar(&mut self, chr: char) {
        self.endlinechar = chr;
    }

    fn get_endlinechar(&self) -> char {
        self.endlinechar
    }
}

impl<L: Iterator<Item = String>> Tokenizer<L> {
    /// Create a new tokenizer over `lines` with default character class assignments.
    pub fn new(lines: L) -> Self {
        let mut map = IntIntervalMap::new(Category::Cat12);

        assign!(map, '\\', Cat0);
        assign!(map, '{', Cat1);
        assign!(map, '}', Cat2);
        assign!(map, '$', Cat3);
        assign!(map, '&', Cat4);
        assign!(map, '\n', Cat5);
        assign!(map, '\r', Cat5);
        assign!(map, '#', Cat6);
        assign!(map, '^', Cat7);
        assign!(map, '_', Cat8);
        assign!(map, '\0', Cat9);
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
        if self.endlinechar as u32 <= 255 {
            line.push(self.endlinechar);
        }
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

    /// Get the catcode of a character
    fn cat(&self, c: char) -> Category {
        self.category_map.get(c as u32)
    }

    /// Parse a superscript-escaped character (e.g. ^^A or ^^0f).
    ///
    /// Returns the replacement character and length of consumed input, if successful
    fn parse_superscript_char(&self) -> Option<(char, usize)> {
        let mut chars = self.input().chars();
        let c_start = match chars.next().filter(|c| self.cat(*c) == Cat7) {
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
}
