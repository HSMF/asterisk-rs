use std::{collections::HashMap, fs::File, io::Write};

use ansi_term::Color;
use anyhow::{bail, Context};
use itertools::Itertools;
use logos::Logos;
use tracing::info;

use crate::{
    frontends::{ocaml::OcamlVisitor, rust::Rust, Format, Frontend, Render},
    generator::Graph,
    grammar::Grammar,
    run_graphviz,
    table::Table,
};

use self::lex::Token;

mod ast;
mod lex;
mod parser;

#[allow(dead_code)]
fn own_grammar() -> Grammar {
    crate::grammar!(
        Grammar:
        Grammar => N "Configs" N "Rules" @ "Spec { rules: v1, configs: v0 }";
        Grammar => N "Rules" @ "Spec { rules: v0.into_iter().rev().collect(), configs: Vec::new() }";
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
        Idents =>  N "Idents" T "Ident" @ "{let mut v = v0; v.push(v1); v}";
        Idents => @ "Vec::new()";
    )
}

#[allow(dead_code)]
fn own_visitor() -> Rust {
    let s = String::from;
    let p = |a, b| (s(a), s(b));
    Rust::new(
        r#"
        use crate::spec::lex::Token;
        use crate::spec::ast::*;"#
            .to_owned(),
        HashMap::from([
            p("Grammar", "Spec"),
            p("Configs", "Vec<(String, String)>"),
            p("Config", "(String, String)"),
            p("Rules", "Vec<Rule>"),
            p("Rule", "Rule"),
            p("CaseList", "Vec<Expansion>"),
            p("Case", "Expansion"),
            p("Idents", "Vec<String>"),
        ]),
        HashMap::from([p("Ident", "String"), p("Literal", "String")]),
        s("Grammar"),
        s("Token"),
    )
    .use_default_for_token()
}

#[tracing::instrument]
#[allow(dead_code)]
pub fn bootstrap(filename: &str) -> anyhow::Result<()> {
    let mut f = File::create(filename).context("could not create output file")?;
    let mut grammar = own_grammar();
    info!("got grammar:\n{grammar}");
    let s0 = grammar.pool_mut().add("S0".to_owned());
    let graph = Graph::make(&grammar, grammar.initial(s0).into_iter().collect());

    {
        let mut f = std::fs::File::create("output/tmp.dot").unwrap();
        writeln!(f, "{}", graph.print(grammar.pool_mut())).unwrap();

        info!("running graphviz");
        run_graphviz(&"output/tmp.dot")?;
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

    writeln!(f, "{}", Render::new(&visitor, &table, &grammar))
        .context("failed to write to file")?;

    visitor.format(filename)?;

    Ok(())
}

#[tracing::instrument]
pub fn parse_string(s: &str) -> anyhow::Result<(Grammar, Box<dyn Frontend>)> {
    let toks = Token::lexer(s).map(|x| x.expect("lexing error"));

    let spec = match parser::parse(toks) {
        Ok(v) => v,
        Err(e) => match e {
            parser::Error::Msg(m) => bail!("{}", m),
            parser::Error::UnexpectedToken {
                expected,
                received,
                state_id,
                remaining_input,
            } => {
                let all_input = Token::lexer(s)
                    .map(|x| x.expect("lexing error"))
                    .collect_vec();
                use std::fmt::Write;
                let mut error_message = String::new();
                writeln!(
                    error_message,
                    "expected one of {{{}}}",
                    expected
                        .into_iter()
                        .map(|tok| tok.map(|x| format!("{x:?}")).unwrap_or("EOF".to_owned()))
                        .format(", ")
                )?;
                writeln!(
                    error_message,
                    "but received {} in state {state_id}",
                    received
                        .clone()
                        .map(|x| format!("{:?}", x))
                        .unwrap_or("EOF".to_owned())
                )?;
                writeln!(error_message, "relevant context:")?;
                let pos = all_input.len() - remaining_input.len() - 1;
                let start = pos.saturating_sub(5);
                let len = pos - start;
                write!(
                    error_message,
                    "{} ",
                    all_input
                        .iter()
                        .skip(start)
                        .take(len)
                        .map(|x| format!("{}", Color::Yellow.paint(format!("{x:?}"))))
                        .format(", ")
                )?;
                write!(
                    error_message,
                    "{} ",
                    Color::Red.paint(format!("{:?}", &received))
                )?;
                writeln!(
                    error_message,
                    "{}",
                    remaining_input
                        .into_iter()
                        .take(10)
                        .map(|x| format!("{}", Color::Yellow.paint(format!("{x:?}"))))
                        .format(", ")
                )?;
                bail!(error_message)
            }
        },
    };

    let mut builder = Grammar::builder();

    let mut non_term_types = HashMap::new();

    for rule in &spec.rules {
        non_term_types.insert(rule.name.to_owned(), rule.typ.to_owned());
    }

    for rule in spec.rules {
        for expansion in rule.expansions {
            let mut prod_builder = builder.prod_builder();
            for tok in expansion.tokens {
                if non_term_types.contains_key(&tok) {
                    prod_builder = prod_builder.non_term(tok);
                } else {
                    prod_builder = prod_builder.term(tok);
                }
            }
            let prod = prod_builder.finish();
            builder = builder.production(rule.name.to_owned(), prod, expansion.code);
        }
    }

    let configs = spec.configs;
    fn find_case_insensitive<'a>(arr: &'a [(String, String)], key: &str) -> Option<&'a str> {
        arr.iter()
            .find(|x| x.0.to_lowercase() == key.to_lowercase())
            .map(|x| x.1.as_str())
    }

    let entry_point = find_case_insensitive(&configs, "entry")
        .unwrap_or("ENTRY")
        .to_owned();

    let token_type = find_case_insensitive(&configs, "type_token")
        .unwrap_or("token")
        .to_owned();

    let language = find_case_insensitive(&configs, "target")
        .expect("missing target field")
        .to_owned();

    let prelude = find_case_insensitive(&configs, "prelude")
        .expect("missing prelude field")
        .to_owned();

    let term_types: HashMap<String, String> = configs
        .iter()
        .filter_map(|(k, v)| {
            let (l, r) = k.split_once('_')?;
            if l.to_lowercase() != "token" {
                return None;
            }

            Some((r.to_owned(), v.to_owned()))
        })
        .collect();

    let grammar = builder.finish(entry_point.to_owned());

    let visitor: Box<dyn Frontend> = match language.as_str() {
        "ocaml" => Box::new(OcamlVisitor::new(
            prelude,
            non_term_types,
            term_types,
            entry_point,
        )),
        "rust" => Box::new(
            Rust::new(prelude, non_term_types, term_types, entry_point, token_type)
                .use_default_for_token(),
        ),
        _ => bail!("unsupported target language: {language}"),
    };

    Ok((grammar, visitor))
}
