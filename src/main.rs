use tex_parser::token::*;

fn main() {
    let tokenizer = Tokenizer::new(
        "this^^5cabc      is some \\test and \\_stuff;        8 spaces!"
            .lines()
            .map(|l| l.to_owned()),
    );
    let result: Vec<Token> = tokenizer.collect();
    eprintln!("{:#?}", result);
}
