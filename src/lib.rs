#[macro_use]
extern crate nom;

use nom::{many0, AtEof, ErrorKind, IResult};

mod interval_map;

#[cfg(test)]
mod test;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Character<C> {
    Cat00(C), // Escape character (/)
    Cat01(C), // Begin group ({)
    Cat02(C), // End group (})
    Cat03(C), // Math shift ($)
    Cat04(C), // Alignment (&)
    Cat05(C), // End of line
    Cat06(C), // Macro parameter (#)
    Cat07(C), // Math superscript (_)
    Cat08(C), // Math subscript (^)
    Cat09(C), // Ignored
    Cat10(C), // Space
    Cat11(C), // Letter
    Cat12(C), // Other (numbers, special characters)
    Cat13(C), // Active character (~)
    Cat14(C), // Start of comment (%)
    Cat15(C), // Invalid input ([DEL])
}

// Escape = "\\"
// GroupStart = "{"
// GroupEnd = "}"
// MathShift = "$"
// AlignmentTab = "&"
// EndOfLine = "\n"
// Parameter = "#"
// Superscript = "_"
// Subscript = "^"
// IgnoredCharacter = ""
// Space = " "
// Letter = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"
// OtherCharacter = "1234567890;:'@\"!,<.>/?[]()"
// ActiveCharacter = "~"
// Comment = "%"
// InvalidCharacter = ""

#[derive(PartialEq, Clone, Debug)]
pub struct State {
    letters: Vec<char>,
    others: Vec<char>,
}

impl Default for State {
    fn default() -> Self {
        State {
            letters: vec!['a', 'b', 'c'],
            others: vec!['1', '@'],
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextileInput<'i> {
    state: State,
    input: &'i str,
}

impl<'i> TextileInput<'i> {
    pub fn new(input: &'i str, state: State) -> Self {
        TextileInput { input, state }
    }
}

impl<'i> AtEof for TextileInput<'i> {
    fn at_eof(&self) -> bool {
        true
    }
}

/*
impl<'i, 's> InputTakeAtPosition for TextileInput<'i, 's> {
    fn split_at_position<P>(&self, predicate: P) -> IResult<Self, Self, u32>
    where
        P: Fn(Self::Item) -> bool,
    {
        match (0..self.input.len()).find(|b| predicate(self.input[*b])) {
            Some(i) => Ok((
                TextileInput::new(
        }
    }
}
*/

pub fn categorize_character<'i>(
    input: TextileInput<'i>,
) -> IResult<TextileInput<'i>, Character<char>> {
    let mut input: TextileInput = input;
    let letter = match input.input.chars().next() {
        Some(l) => l,
        None => return Err(nom::Err::Error(error_position!(input, ErrorKind::Tag))),
    };
    if input.state.letters.contains(&letter) {
        return Ok((
            TextileInput::new(&input.input[1..], input.state),
            Character::Cat11(letter),
        ));
    }

    if input.state.others.contains(&letter) {
        if letter == '1' {
            input.state.others.push('a');
            input.state.letters.remove(0);
        }
        return Ok((
            TextileInput::new(&input.input[1..], input.state),
            Character::Cat12(letter),
        ));
    }

    Err(nom::Err::Error(error_position!(input, ErrorKind::Tag)))
}

pub fn categorize_string<'i>(
    input: TextileInput<'i>,
) -> IResult<TextileInput<'i>, Vec<Character<char>>> {
    many0!(input, categorize_character)
}
