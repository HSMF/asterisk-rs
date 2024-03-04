#[derive(Debug, Clone)]
pub enum Item {
    Assn(String, Box<Expr>),
    Func(String, Vec<String>, Vec<Item>),
    Return(Box<Expr>),
}

#[derive(Debug, Clone)]
pub enum Expr {
    Int(i32),
    Var(String),
    Bop(Bop, Box<Expr>, Box<Expr>),
    Uop(Uop, Box<Expr>),
    Call(String, Vec<Expr>),
    If(Box<Expr>, Vec<Item>, Vec<Item>),
}

#[derive(Debug, Clone, Copy)]
pub enum Bop {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug, Clone, Copy)]
pub enum Uop {
    Pos,
    Neg,
}
