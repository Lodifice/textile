use tex_parser::char::*;
use tex_parser::token::*;

fn main() {
    let tokenizer = Tokenizer::new("this^^5cabc is some \\test and \\_stuff");
    let result: Vec<Token> = tokenizer.collect();
    eprintln!("{:?}", result);
}
