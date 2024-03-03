use std::collections::HashMap;

use anyhow::{bail, Context};
use logos::Logos;

use crate::{
    frontends::{
        rust::{format_src, Rust},
        Render,
    },
    generator::Graph,
    grammar::Grammar,
    run_graphviz,
    table::Table,
};

use self::lex::Token;

mod ast;
mod lex;

#[allow(dead_code)]
fn own_grammar() -> Grammar {
    crate::grammar!(
        Grammar:
        Grammar => N "Configs" N "Rules" @ "Spec { rules: v1, configs: v0 }";
        Grammar => N "Rules" @ "Spec { rules: v0, configs: Vec::new() }";
        Configs => N "Configs" N "Config" @ "{let mut v0 = v0; v0.push(v1); v0}";
        Configs => N "Config" @ "vec![v0]";
        Config => T "Ident" T "Equals" T "Ident" @ "(v0, v2)";
        Config => T "Ident" T "Equals" T "Literal" @ "(v0, v2)";
        Rules => N "Rule" N "Rules" @ "{let mut v = v1; v.push(v0); v}";
        Rules => @ "Vec::new()";
        Rule => T "Ident" T "Colon" T "Literal" N "CaseList" @ "Rule {name: v0, typ: v2, expansions: v3}";
        CaseList => N "Case" N "CaseList" @ "{let mut v = v1; v.push(v0); v}";
        CaseList => @ "Vec::new()";
        Case => T "Pipe" N "Idents" T "Literal" @ "Expansion {tokens: v1, code: v2}";
        Idents => T "Ident" N "Idents" @ "{let mut v = v1; v.push(v0); v}";
        Idents => @ "Vec::new()";
    )
}

#[allow(dead_code)]
fn own_visitor() -> Rust {
    let s = String::from;
    let p = |a, b| (s(a), s(b));
    Rust::new(
        "use lex::Token;".to_owned(),
        HashMap::from([
            p("A", "i32"),
            p("Grammar", "Spec"),
            p("Configs", "Vec<(String, String)>"),
            p("Config", "(String, String)"),
            p("Rules", "Vec<Rule>"),
            p("Rule", "Rule"),
            p("CaseList", "Vec<Expansion>"),
            p("Case", "Expansion"),
            p("Idents", "Vec<String>"),
        ]),
        HashMap::from([p("Ident", "String"), p("Literal", "String"), p("B", "i32")]),
        s("Grammar"),
        s("Token"),
    )
    .use_default_for_token()
}

#[allow(dead_code)]
pub fn bootstrap(mut f: impl std::io::Write) -> anyhow::Result<()> {
    let mut grammar = own_grammar();
    println!("{grammar}");
    let s0 = grammar.pool_mut().add("S0".to_owned());
    let graph = Graph::make(&grammar, grammar.initial(s0).into_iter().collect());

    {
        use std::io::Write;
        let mut f = std::fs::File::create("output/tmp.dot").unwrap();
        writeln!(f, "{}", graph.print(grammar.pool_mut())).unwrap();

        eprintln!("spec: running graphviz");
        run_graphviz("output/tmp.dot")?;
    }

    let table = match Table::from_graph(&graph) {
        Ok(t) => t,
        Err(conflict) => {
            bail!(
                "could not construct table because of conflict in {}: token={}  \neither: {}\nor:     {}",
                conflict.state,
                conflict.token.display(grammar.pool()),
                conflict.either.display(grammar.pool()),
                conflict.or.display(grammar.pool())
            );
        }
    };

    let visitor = own_visitor();

    writeln!(f, "{}", Render::new(visitor, &table, &grammar)).context("failed to write to file")?;

    Ok(())
}

pub fn parse_string(s: &str) {
    let toks = Token::lexer(s).map(|x| x.expect("lexing error"));
    for tok in toks {
        println!("{tok:?}");
    }
}
