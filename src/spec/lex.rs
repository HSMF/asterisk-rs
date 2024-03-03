use logos::{Lexer, Logos};

fn parse_literal(lex: &mut Lexer<Token>) -> Option<String> {
    let remainder = lex.remainder();
    let mut depth = 1;
    for (i, c) in remainder.char_indices() {
        match c {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth <= 0 {
                    lex.bump(i + 1);
                    return Some(remainder[..i].to_owned());
                }
            }
            _ => {}
        }
    }
    None
}

#[derive(Logos, Debug, PartialEq, Eq, Clone)]
#[logos(skip r"[ \t\n\f]+")]
#[logos(skip r"#[^\n]*\n")]
pub enum Token {
    #[token("=")]
    Equals,

    #[token(":")]
    Colon,

    #[token("|")]
    Pipe,

    #[regex("[a-zA-Z_][a-zA-Z_0-9]+", |l| Some(l.slice().to_owned()))]
    Ident(String),

    #[token("{", parse_literal)]
    Literal(String),
}
