use std::path::PathBuf;
use std::{io::Write, path::Path};

use anyhow::{anyhow, Context};
use clap::Parser;
use grammar::Grammar;

use crate::{frontends::Render, generator::Graph, spec::parse_string, table::Table};

mod frontends;

mod generator;
mod grammar;
mod spec;
mod string_pool;
mod table;

#[derive(clap::Parser)]
#[clap(version, author)]
struct Cli {
    #[clap(short = 'O', long)]
    output_dir: Option<String>,

    #[clap(short, long)]
    output: Option<String>,

    #[clap(short, long)]
    emit_dot: bool,

    #[clap(short, long)]
    bootstrap: bool,

    grammar: String,
}

pub fn run_graphviz<P>(path: &P) -> anyhow::Result<()>
where
    P: AsRef<Path>,
{
    let output = PathBuf::from(path.as_ref()).with_extension("svg");
    let mut handle = std::process::Command::new("dot")
        .arg("-Tsvg")
        .arg("-Gfontname=monospace")
        .arg("-Efontname=monospace")
        .arg("-Nfontname=monospace")
        .arg(path.as_ref())
        .arg("-o")
        .arg(output)
        .spawn()
        .context("couldnt spawn graphviz")?;

    handle.wait()?;
    Ok(())
}

fn main() -> anyhow::Result<()> {
    // let grammar = grammar!(
    // "A" => T "(" N "B" @ "hello world";
    // "B" => T "(" N "A" @ "hello world";
    // );

    let mut grammar = grammar!(
        A:
        A => N "B" T "Plus" N "A"  @ "v0 + v2";
        A => N "B" @ "v0";
        B => N "C" T "Mul" N "B" @ "v0 * v2";
        B => N "C" @ "v0";
        C => T "OpenParen" N "A" T "CloseParen" @ "v1";
        C => T "Int" @ "v0";
    );

    println!("{grammar}");

    let a = grammar.pool_mut().add("S0".to_owned());
    let _graph = Graph::make(&grammar, grammar.initial(a).into_iter().collect());
    // use std::io::Write;
    // let mut f = std::fs::File::create("output/tmp.dot").unwrap();
    // writeln!(f, "{}", graph.print(grammar.pool_mut())).unwrap();
    //
    // eprintln!("running graphviz");
    //
    // let table = Table::from_graph(&graph).expect("could construct table");
    // println!("{:?}", grammar.pool());
    // // println!("{table:?}");
    //
    // let v = OcamlVisitor::new(
    //     "type token = Int of int | Plus | Mul | OpenParen | CloseParen".to_owned(),
    //     HashMap::from([
    //         ("A".to_owned(), "int".to_owned()),
    //         ("B".to_owned(), "int".to_owned()),
    //         ("C".to_owned(), "int".to_owned()),
    //     ]),
    //     HashMap::from([("Int".to_owned(), "int".to_owned())]),
    // );
    // let mut f = std::fs::File::create("output/hey.ml").unwrap();
    // writeln!(f, "{}", Render::new(v, &table, &grammar)).unwrap();
    //
    // let v = Rust::new(
    //     // "#[derive(Clone, Debug)] pub enum Token{Int(i32), Plus, Mul, OpenParen, CloseParen}"
    //     //     .to_owned(),
    //     "use super::Token;".to_owned(),
    //     HashMap::from([
    //         ("A".to_owned(), "i32".to_owned()),
    //         ("B".to_owned(), "i32".to_owned()),
    //         ("C".to_owned(), "i32".to_owned()),
    //     ]),
    //     HashMap::from([("Int".to_owned(), "i32".to_owned())]),
    //     "A".to_owned(),
    //     "Token".to_owned(),
    // )
    // .use_default_for_token();
    // let mut f = std::fs::File::create("temp/src/parser.rs").unwrap();
    // writeln!(f, "{}", Render::new(v, &table, &grammar)).unwrap();
    // let mut handle = std::process::Command::new("rustfmt")
    //     .arg("temp/src/parser.rs")
    //     .spawn()
    //     .expect("could spawn rustfmt");
    // handle.wait().expect("could not wait for rustfmt");

    // let v = OcamlVisitor::new("(* hello world, this is the prelude *)".to_owned());
    // println!("{}", Render::new(v, &table));

    let cli = Cli::parse();

    if cli.bootstrap {
        spec::bootstrap(&cli.grammar).context("could not bootstrap")?;

        return Ok(());
    }

    let grammar = std::fs::read_to_string(cli.grammar).expect("failed to read grammar");
    let (grammar, visitor) = parse_string(&grammar)?;
    let graph = Graph::make(&grammar, grammar.initial(a).into_iter().collect());

    if cli.emit_dot {
        let output_dir = cli
            .output_dir
            .ok_or(anyhow!("output dir must be set to emit graphviz"))?;
        let mut p = PathBuf::from(output_dir);
        p.push("dfa.dot");
        let mut f = std::fs::File::create(&p).unwrap();
        writeln!(f, "{}", graph.print(grammar.pool())).unwrap();

        run_graphviz(&p).context("failed to run graphviz")?;
    }

    let table = Table::from_graph(&graph).expect("could construct table");

    if let Some(output) = cli.output {
        let mut f = std::fs::File::create(&output).unwrap();
        writeln!(f, "{}", Render::new(&visitor, &table, &grammar)).unwrap();
        visitor.format(&output).context("failed to format")?;
    } else {
        println!("{}", Render::new(&visitor, &table, &grammar));
    }

    Ok(())
}
