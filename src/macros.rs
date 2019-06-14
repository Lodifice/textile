use crate::token::*;
/// Implements a TeX expansion processor.
use std::error::Error;

/// A location in the input file.
#[derive(Debug, Clone)]
pub struct Span {
    /// Line *number* and column *index* of the location start.
    pub start: (usize, usize),
    /// Line *number* and column *index* of the location end.
    pub end: (usize, usize),
}

impl Span {
    pub fn new(start: (usize, usize), end: (usize, usize)) -> Self {
        Span { start, end }
    }

    pub fn extend_to(&mut self, end: (usize, usize)) {
        self.end = end;
    }

    pub fn any() -> Self {
        Span {
            start: (0, 0),
            end: (0, 0),
        }
    }
}

impl PartialEq for Span {
    fn eq(&self, other: &Span) -> bool {
        (self.start == other.start && self.end == other.end)
            || (self.start == (0, 0) && self.end == (0, 0))
    }
}

/// Parameters of a macro.
/// Can be undelimited or delimited.
#[derive(Debug, Clone, PartialEq)]
enum MacroParameter {
    Undelimited(u8),
    Delimited(u8, Vec<Token>),
}

/// Represents the definition of a TeX macro.
#[derive(Debug, Clone, PartialEq)]
pub struct Macro {
    /// The control sequence name of this macro.
    control_sequence: String,
    /// Parameter token list.
    parameters: Vec<MacroParameter>,
    /// Output token list.
    replacement_text: Vec<Token>,
    /// Where the macro was defined
    location: Span,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ExpansionError {
    InvalidDefName,
    ExplicitBracesInParameterText,
    NonConsequitiveParameterNumber,
}

impl std::fmt::Display for ExpansionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExpansionError::InvalidDefName => write!(f, "Invalid Definition Name"),
            ExpansionError::ExplicitBracesInParameterText => {
                write!(f, "Explicit Braces in Macro Parameter Text")
            }
            ExpansionError::NonConsequitiveParameterNumber => {
                write!(f, "Non-Consequtive Parameter Number in Parameter Text")
            }
        }
    }
}

impl Error for ExpansionError {
    fn description(&self) -> &str {
        match self {
            ExpansionError::InvalidDefName => {
                "The first argument of a macro definition must be a control sequence!"
            }
            ExpansionError::ExplicitBracesInParameterText => {
                "The macro parameter text cannot contain explicit groups!"
            }
            ExpansionError::NonConsequitiveParameterNumber => {
                "Macro parameters must be numbered consequtively!"
            }
        }
    }

    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ExpansionError::InvalidDefName => None,
            ExpansionError::ExplicitBracesInParameterText => None,
            ExpansionError::NonConsequitiveParameterNumber => None,
        }
    }
}

impl Macro {
    /// Parse the parameter list.
    fn parse_param_list(param_text: Vec<Token>) -> Result<Vec<MacroParameter>, ExpansionError> {
        let mut delimiter = vec![];
        let mut param_number = None;
        let mut arg_start = false;
        let mut params = vec![];

        let mut finish_param = |number, delim_vec: &mut Vec<Token>| {
            // undelimited parameters
            if delim_vec.is_empty() {
                params.push(MacroParameter::Undelimited(number));
            // delimited parameters have a token list which delimits them
            // from the previous parameters
            } else {
                params.push(MacroParameter::Delimited(
                    number,
                    delim_vec.drain(..).collect(),
                ));
            }
        };
        for token in param_text {
            match (param_number, token) {
                (None, Token::Character(_, Category::Cat6)) => {
                    arg_start = true;
                    continue;
                }
                (Some(number), Token::Character(_, Category::Cat6)) => {
                    finish_param(number, &mut delimiter);
                    param_number = None;
                    arg_start = true;
                    continue;
                }
                (None, Token::Character(c, _)) if c.is_ascii_digit() && arg_start => {
                    param_number = Some(((c as u32) - 48) as u8)
                }
                (_, Token::Character(_, Category::Cat1))
                | (_, Token::Character(_, Category::Cat2)) => {
                    return Err(ExpansionError::ExplicitBracesInParameterText)
                }
                (_, token) => delimiter.push(token),
            }
            arg_start = false;
        }
        if let Some(number) = param_number {
            finish_param(number, &mut delimiter)
        }
        Ok(params)
    }

    pub fn define(
        control_sequence: Token,
        parameter_text: Vec<Token>,
        replacement_text: Vec<Token>,
    ) -> Result<Macro, ExpansionError> {
        let (name, def_start) = match control_sequence {
            Token::ControlSequence(name, span) => (name, (span.line, span.start)),
            _ => return Err(ExpansionError::InvalidDefName),
        };

        let params = Self::parse_param_list(parameter_text)?;
        eprintln!("{:?}", params);

        let def_end = (0, 0);
        Ok(Macro {
            control_sequence: name,
            parameters: params,
            replacement_text,
            location: Span {
                start: def_start,
                end: def_end,
            },
        })
    }
}

#[cfg(test)]
mod expansion_test {
    use crate::macros::*;
    use crate::token::{Category::*, OtherToken::*, Token::*, *};

    fn tokens(input: &str) -> Vec<Token> {
        let mut tokenizer = Tokenizer::new(input.lines().map(|s| s.to_owned()));
        // disable endlinechar
        tokenizer.set_endlinechar(std::char::from_u32(256).unwrap());
        tokenizer.collect()
    }

    #[test]
    fn define_macro() {
        let cs = ControlSequence("test".to_owned(), crate::token::Span::any());
        let param = vec![];
        let replacement = tokens("hello world!");
        assert!(Macro::define(cs, param, replacement).is_ok());
    }

    #[test]
    fn define_macro_with_args() {
        let cs = ControlSequence("PickTwo".to_owned(), crate::token::Span::any());
        let param = tokens("#1abc#2");
        let replacement = tokens("(#1,#2)");
        assert_eq!(
            Err(ExpansionError::ExplicitBracesInParameterText),
            Macro::define(cs.clone(), tokens("#1{#2}"), replacement.clone())
        );
        assert_eq!(
            Macro {
                control_sequence: "PickTwo".into(),
                parameters: vec![
                    MacroParameter::Delimited(1, tokens("abc")),
                    MacroParameter::Undelimited(2)
                ],
                replacement_text: replacement.clone(),
                location: crate::macros::Span::any()
            },
            Macro::define(cs, param, replacement).expect("could not define macro!")
        );
    }
}
