use std::{collections::HashMap, fmt::Display};

use itertools::Itertools;

use crate::ast::{Bop, Expr, Item, Uop};

macro_rules! diverge {
    ($e:expr) => {{
        let (e, diverges) = $e;
        if diverges {
            return (e, true);
        }
        e
    }};
}

#[derive(Debug, Clone)]
enum Value {
    Int(i32),
    Func(Vec<String>, Vec<Item>),
}

impl Value {
    fn int(self) -> Option<i32> {
        match self {
            Value::Int(i) => Some(i),
            _ => None,
        }
    }

    fn func(&self) -> Option<(&[String], &[Item])> {
        match self {
            Value::Func(a, b) => Some((a, b)),
            _ => None,
        }
    }
}

#[allow(dead_code)]
fn print_ctx(ctx: &HashMap<String, Value>) {
    print!("{{");
    for (k, v) in ctx {
        print!("{k:?} -> {v}, ");
    }
    println!("}}");
}

fn is_builtin(name: &str) -> bool {
    matches!(name, "print" | "println" | "printc")
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Int(i) => write!(f, "{i}"),
            Value::Func(_, _) => write!(f, "<function>"),
        }
    }
}

fn eval_exp(ctx: &HashMap<String, Value>, exp: &Expr) -> (Value, bool) {
    match exp {
        Expr::Int(i) => (Value::Int(*i), false),
        Expr::Var(v) => (
            (ctx.get(v.as_str()).expect("undefined variable")).to_owned(),
            false,
        ),
        Expr::Bop(bop, l, r) => {
            let l = diverge!(eval_exp(ctx, l));
            let r = diverge!(eval_exp(ctx, r));
            let l = l.int().expect("type error");
            let r = r.int().expect("type error");
            (
                match bop {
                    Bop::Add => Value::Int(l + r),
                    Bop::Sub => Value::Int(l - r),
                    Bop::Mul => Value::Int(l * r),
                    Bop::Div => Value::Int(l / r),
                },
                false,
            )
        }
        Expr::Uop(uop, exp) => {
            let e = diverge!(eval_exp(ctx, exp));
            let e = e.int().expect("type error");
            (
                match uop {
                    Uop::Pos => Value::Int(e),
                    Uop::Neg => Value::Int(-e),
                },
                false,
            )
        }
        Expr::Call(name, args) if is_builtin(name) => {
            let args = args.iter().map(|x| eval_exp(ctx, x)).collect_vec();
            if let Some(v) = args.iter().find(|x| x.1) {
                return (v.0.clone(), true);
            }
            match name.as_str() {
                "print" => {
                    print!("{}", args.into_iter().map(|x| x.0).format(" "));
                    (Value::Int(0), false)
                }
                "println" => {
                    println!("{}", args.into_iter().map(|x| x.0).format(" "));
                    (Value::Int(0), false)
                }
                "printc" => {
                    print!(
                        "{}",
                        args.into_iter()
                            .map(|x| x.0.int().expect("type error: expected int"))
                            .map(|x| char::from_u32(x as u32)
                                .unwrap_or(char::REPLACEMENT_CHARACTER))
                            .format("")
                    );
                    (Value::Int(0), false)
                }
                _ => unreachable!(),
            }
        }
        Expr::Call(name, args) => {
            let (names, func) = ctx
                .get(name.as_str())
                .or_else(|| {
                    eprintln!("failed to get {name}");
                    None
                })
                .expect("undefined function")
                .func()
                .expect("type error");
            let mut child_ctx = ctx.clone();
            assert_eq!(
                args.len(),
                names.len(),
                "must call with the same amount of arguments"
            );
            for (arg, name) in args.iter().zip(names.iter()) {
                child_ctx.insert(name.to_owned(), diverge!(eval_exp(ctx, arg)));
            }
            (interpret_func(child_ctx, func), false)
        }
        Expr::If(cond, yes, no) => {
            let cond = diverge!(eval_exp(ctx, cond)).int().expect("type error");
            // eprintln!("COND IS: {cond} {ctx:?}");
            if cond != 0 {
                interpret_items(ctx.clone(), yes)
            } else {
                interpret_items(ctx.clone(), no)
            }
        }
    }
}

fn interpret_func(ctx: HashMap<String, Value>, prog: &[Item]) -> Value {
    interpret_items(ctx, prog).0
}

fn interpret_items(mut ctx: HashMap<String, Value>, prog: &[Item]) -> (Value, bool) {
    for item in prog {
        match item {
            Item::Assn(id, exp) => {
                let value = diverge!(eval_exp(&ctx, exp));
                ctx.insert(id.to_owned(), value);
            }
            Item::Func(name, args, body) => {
                ctx.insert(name.to_owned(), Value::Func(args.clone(), body.clone()));
            }
            Item::Return(exp) => {
                let value = diverge!(eval_exp(&ctx, exp));
                return (value, true);
            }
        }
    }

    (Value::Int(0), false)
}

pub fn interpret(prog: &[Item]) {
    let ctx = HashMap::new();
    interpret_items(ctx, prog);
}
