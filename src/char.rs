use crate::interval_map::{IntIntervalMap, IntervalMap};
use nom::{many0, AtEof, ErrorKind, IResult};

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

macro_rules! map_cats {
    ($u: ident, [$($cat:ident),*], $f: ident) => {
        match $u {
            $(
                Character::$cat(v) => Character::$cat($f(v))
            ),*
        }
    }
}

impl<U> Character<U> {
    pub fn map<V>(self, f: &Fn(U) -> V) -> Character<V> {
        map_cats!(
            self,
            [
                Cat00, Cat01, Cat02, Cat03, Cat04, Cat05, Cat06, Cat07, Cat08, Cat09, Cat10, Cat11,
                Cat12, Cat13, Cat14, Cat15
            ],
            f
        )
    }
}

#[derive(PartialEq, Clone, Debug)]
pub struct State {
    category_map: IntIntervalMap<u32, Character<()>>,
}

macro_rules! assign {
    ($map:ident, $lo:literal, $hi:literal, $cls:ident) => {
        $map.assign(($lo as u32)..($hi as u32) + 1, Character::$cls(()));
    };
    ($map:ident, $idx:literal, $cls:ident) => {
        $map.assign_single($idx as u32, Character::$cls(()));
    };
}
impl Default for State {
    fn default() -> Self {
        let mut map = IntIntervalMap::new(Character::Cat14(()));

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

pub fn categorize_character<'i>(
    input: TextileInput<'i>,
) -> IResult<TextileInput<'i>, Character<char>> {
    let letter = match input.input.chars().next() {
        Some(l) => l,
        None => return Err(nom::Err::Error(error_position!(input, ErrorKind::Tag))),
    };
    let out = input.state.category_map.get(letter as u32).map(&|_| letter);
    Ok((TextileInput::new(&input.input[1..], input.state), out))
}

pub fn categorize_string<'i>(
    input: TextileInput<'i>,
) -> IResult<TextileInput<'i>, Vec<Character<char>>> {
    many0!(input, categorize_character)
}
