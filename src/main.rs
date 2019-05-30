#[derive(Debug)]
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

#[macro_use]
extern crate nom;

use nom::{ErrorKind, IResult};

static mut letters: Option<Vec<char>> = None;
static mut others: Option<Vec<char>> = None;

unsafe fn letters_foo() -> &'static mut Vec<char> {
    if letters.is_none() {
        letters = Some(vec!['a', 'b', 'c']);
    }
    letters.as_mut().unwrap()
}

unsafe fn others_foo() -> &'static mut Vec<char> {
    if others.is_none() {
        others = Some(vec!['1', '@']);
    }
    others.as_mut().unwrap()
}

fn categorize_character(input: &str) -> IResult<&str, Character<char>> {
    unsafe {
        if letters_foo().contains(&input.chars().next().unwrap()) {
            Ok((&input[1..], Character::Cat11(input.chars().next().unwrap())))
        } else if others_foo().contains(&input.chars().next().unwrap()) {
            if input.chars().next().unwrap() == '1' {
                others_foo().push('a');
                letters_foo().remove(0);
            }
            Ok((&input[1..], Character::Cat12(input.chars().next().unwrap())))
        } else {
            Err(nom::Err::Error(error_position!(&input[0..], ErrorKind::Tag)))
        }
    }
}

fn categorize_string(input: &str) -> IResult<&str, Vec<Character<char>>> {
    many0!(input, categorize_character)
}

fn main() {
    println!("{:?}", categorize_character("a"));
    println!("{:?}", categorize_string("abba1@a#"));
}

                 
