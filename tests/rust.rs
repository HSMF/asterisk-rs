use std::process::Command;

use common::{asterisk_gen, build_dir};

use crate::common::CommandExt;

mod common;

#[test]
fn parens() -> anyhow::Result<()> {
    let build = build_dir();
    Command::new("cargo")
        .arg("init")
        .arg("--quiet")
        .arg("--name")
        .arg("parens")
        .current_dir(build.path())
        .run()?;
    let file_path = build.path().join("src").join("parser.rs");

    asterisk_gen(file_path, "./tests/frontends/rust/parens.ast").run()?;
    std::fs::copy(
        "./tests/frontends/rust/parens.rs",
        build.path().join("src").join("main.rs"),
    )?;

    Command::new("cargo")
        .env("RUSTFLAGS", "-Awarnings")
        .arg("run")
        .arg("--quiet")
        .current_dir(build.path())
        .run()?;

    Ok(())
}
