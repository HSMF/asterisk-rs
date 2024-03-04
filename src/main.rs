use std::path::PathBuf;
use std::{io::Write, path::Path};

use anyhow::{anyhow, bail, Context};
use clap::Parser;
use tracing::{info, warn};

use crate::{frontends::Render, generator::Graph, spec::parse_string, table::Table};

mod frontends;

mod generator;
mod grammar;
mod spec;
mod string_pool;
mod table;

#[derive(clap::Parser, Debug)]
#[clap(version, author)]
struct Cli {
    /// sets the output directory for artifacts
    ///
    /// must be set when --emit-dot is provided
    #[clap(short = 'O', long)]
    output_dir: Option<String>,

    /// sets the output path for the generated code file. If this is not provided, all output is
    /// written to stdout
    #[clap(short, long)]
    output: Option<String>,

    /// whether to run a formatter on the generated code
    ///
    /// Uses `rustfmt` for rust, `ocamlformat` for ocaml
    #[clap(short, long)]
    format: bool,

    /// emit a graph representation of the parsing dfa for debugging
    #[clap(short, long)]
    emit_dot: bool,

    /// bootstrap the grammar
    ///
    /// If bootstrap is set, the positional argument becomes the input.
    ///
    /// Usage here is asterisk-rs --bootstrap path/to/new/grammar.rs
    /// This should only be used by `asterisk-rs` itself
    #[clap(short, long)]
    bootstrap: bool,

    /// Input file where all the definitions are placed.
    ///
    /// For informations on the syntax, consider the relevant documentation
    grammar: String,
}

#[tracing::instrument]
pub fn run_graphviz<P>(path: &P) -> anyhow::Result<()>
where
    P: AsRef<Path> + std::fmt::Debug,
{
    let output = PathBuf::from(path.as_ref()).with_extension("svg");
    info!(
        "running graphviz command: {} -> {}",
        path.as_ref().display(),
        output.display()
    );
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
    let cli = Cli::parse();
    tracing_subscriber::fmt::init();

    if cli.bootstrap {
        warn!("--bootstrap is to bootstrap asterisk-rs itself. is this really what you want?");
        spec::bootstrap(&cli.grammar).context("could not bootstrap")?;

        return Ok(());
    }

    info!("reading {:?} as grammar file", cli.grammar);
    let grammar = std::fs::read_to_string(&cli.grammar).expect("failed to read grammar");
    let (grammar, visitor) = parse_string(&grammar)?;
    let a = grammar
        .pool()
        .get_reverse("S0")
        .expect("S0 should be in grammar");
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

    if let Some(output) = cli.output {
        let mut f = std::fs::File::create(&output).unwrap();
        info!("writing to {output}");
        writeln!(f, "{}", Render::new(&visitor, &table, &grammar)).unwrap();

        if cli.format {
            visitor.format(&output).context("failed to format")?;
        }
    } else {
        println!("{}", Render::new(&visitor, &table, &grammar));
    }

    Ok(())
}
