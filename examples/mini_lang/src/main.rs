use logos::Logos;

use crate::interpret::interpret;

mod ast;
mod interpret;
mod parser;

#[derive(logos::Logos, Clone, Debug, PartialEq)]
#[logos(skip r"[ \t\n\f]+")]
pub enum Token {
    #[token("(")]
    OpenParen,

    #[token(")")]
    CloseParen,

    #[token("*")]
    Mul,
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("/")]
    Div,

    #[token("=")]
    Equals,
    #[token(",")]
    Comma,

    #[token("begin")]
    Begin,
    #[token("end")]
    End,
    #[token("if")]
    If,
    #[token("then")]
    Then,
    #[token("else")]
    Else,
    #[token("return")]
    Return,

    #[regex(r"[0-9]+", |lex| lex.slice().parse().ok())]
    Int(i32),

    #[regex(r"[a-zA-Z_][a-zA-Z_0-9]*", |lex| lex.slice().to_owned())]
    Ident(String),
}

fn main() {
    // let arg: String =
    //     Itertools::intersperse(std::env::args().skip(1), String::from("\n")).collect();
    let arg = include_str!("./prog");
    let tokens = Token::lexer(arg)
        .map(|x| x.unwrap())
        // .inspect(|tok| println!("{tok:?}"))
    ;
    let abc = parser::parse(tokens).expect("parse error");
    // println!("{abc:#?}");
    interpret(&abc);
}
