use crate::char::Category;
use crate::char::Category::*;
use crate::interval_map::{IntIntervalMap, IntervalMap};
use std::char::from_u32;

/// Tokens as described in chapter 7 of the texbook
#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    ControlSequence(String),
    Character(char, Category),
    Comment(String),
    Ignored(char),
}

/// The tokenizer states as described in chapter 8 of the texbook
#[derive(Debug, PartialEq, Clone)]
pub enum TokenizerState {
    LineStart,
    LineMiddle,
    SkippingBlanks,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Tokenizer<'i> {
    category_map: IntIntervalMap<u32, Category>,
    state: TokenizerState,
    input: &'i str,
}

impl<'i> Iterator for Tokenizer<'i> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        let chr = match self.pop_char() {
            Some(c) => c,
            None => return None,
        };
        let cat = self.cat(chr);

        // process state as described in chapter 8, p. 46 of the texbook
        match cat {
            Cat00 => match self.pop_char() {
                None => Some(Token::ControlSequence(String::new())),
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
                    Some(Token::ControlSequence(content))
                }
            },
            Cat01 | Cat02 | Cat03 | Cat04 | Cat06 | Cat07 | Cat08 | Cat11 | Cat12 | Cat13 => {
                self.state = TokenizerState::LineMiddle;
                Some(Token::Character(chr, cat))
            }
            _ => None,
        }
    }
}

macro_rules! assign {
    ($map:ident, $lo:literal, $hi:literal, $cls:ident) => {
        $map.assign(($lo as u32)..($hi as u32) + 1, Category::$cls);
    };
    ($map:ident, $idx:literal, $cls:ident) => {
        $map.assign_single($idx as u32, Category::$cls);
    };
}

impl<'i> Tokenizer<'i> {
    /// pop the next character from the input.
    /// the character might have been esacped,
    /// which consumes more input than one character.
    fn pop_char(&mut self) -> Option<char> {
        match self.parse_superscript_char() {
            Some((c, l)) => {
                self.input = &self.input[l..];
                Some(c)
            }
            None => match self.input.chars().next() {
                Some(c) => {
                    self.input = &self.input[1..];
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
            None => self.input.chars().next(),
        }
    }

    fn cat(&self, c: char) -> Category {
        self.category_map.get(c as u32)
    }
    /// Parse a superscript-escaped character (e.g. ^^A or ^^0f).
    ///
    /// Returns the replacement character and length of consumed input, if successful
    fn parse_superscript_char(&self) -> Option<(char, usize)> {
        let mut chars = self.input.chars();
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
                    u8::from_str_radix(&self.input[2..4], 16)
                        .expect("parse error with superscript-escaped hex character")
                        as u32,
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
        return None;
    }

    pub fn new(input: &'i str) -> Self {
        let mut map = IntIntervalMap::new(Category::Cat14);

        assign!(map, '\\', Cat00);
        assign!(map, '{', Cat01);
        assign!(map, '}', Cat02);
        assign!(map, '$', Cat03);
        assign!(map, '&', Cat04);
        assign!(map, '\n', Cat05);
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
        assign!(map, '\x0b', '\x1f', Cat15);

        Tokenizer {
            category_map: map,
            state: TokenizerState::LineStart,
            input,
        }
    }
}
