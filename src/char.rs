use crate::interval_map::{IntIntervalMap, IntervalMap};
use nom::{many0, AtEof, ErrorKind, IResult};

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

type Character = (Category, char);

#[derive(PartialEq, Clone, Debug)]
pub struct State {
    category_map: IntIntervalMap<u32, Category>,
}

macro_rules! assign {
    ($map:ident, $lo:literal, $hi:literal, $cls:ident) => {
        $map.assign(($lo as u32)..($hi as u32) + 1, Category::$cls);
    };
    ($map:ident, $idx:literal, $cls:ident) => {
        $map.assign_single($idx as u32, Category::$cls);
    };
}
impl Default for State {
    fn default() -> Self {
        let mut map = IntIntervalMap::new(Category::Cat14);

        assign!(map, '\\', Cat00);
        assign!(map, '{', Cat01);
        assign!(map, '}', Cat02);
        assign!(map, '$', Cat03);
        assign!(map, '&', Cat04);
        assign!(map, '\n', Cat05);
        assign!(map, '#', Cat06);
        assign!(map, '_', Cat07);
        assign!(map, '^', Cat08);
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

        State { category_map: map }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextileInput<'i> {
    pub state: State,
    pub input: &'i str,
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

pub fn categorize_character<'i>(input: TextileInput<'i>) -> IResult<TextileInput<'i>, Character> {
    let letter = match input.input.chars().next() {
        Some(l) => l,
        None => return Err(nom::Err::Error(error_position!(input, ErrorKind::Tag))),
    };
    let out = (input.state.category_map.get(letter as u32), letter);
    Ok((TextileInput::new(&input.input[1..], input.state), out))
}

pub fn categorize_string<'i>(input: TextileInput<'i>) -> IResult<TextileInput<'i>, Vec<Character>> {
    many0!(input, categorize_character)
}
