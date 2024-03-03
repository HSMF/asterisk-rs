use logos::{Lexer, Logos};

fn parse_literal<'a>(lex: &mut Lexer<'a, Token<'a>>) -> Option<&'a str> {
    let remainder = lex.remainder();
    let mut depth = 1;
    for (i, c) in remainder.char_indices() {
        match c {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth <= 0 {
                    lex.bump(i + 1);
                    return Some(&remainder[..i]);
                }
            }
            _ => {}
        }
    }
    None
}

#[derive(Logos, Debug, PartialEq, Eq)]
#[logos(skip r"[ \t\n\f]+")]
#[logos(skip r"#[^\n]*\n")]
pub enum Token<'a> {
    #[token("=")]
    Equals,

    #[token(":")]
    Colon,

    #[token("|")]
    Pipe,

    #[regex("[a-zA-Z_][a-zA-Z_0-9]+")]
    Ident(&'a str),

    #[token("{", parse_literal)]
    Literal(&'a str),
}
