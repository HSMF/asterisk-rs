use std::process::Command;

use common::{asterisk_gen, build_dir};

use crate::common::CommandExt;

mod common;

#[test]
fn parens() -> anyhow::Result<()> {
    let build = build_dir();
    let file_path = build.path().join("parser.py");

    asterisk_gen(file_path, "./tests/frontends/python/parens.ast").run()?;
    std::fs::copy(
        "./tests/frontends/python/parens.py",
        build.path().join("main.py"),
    )?;
    std::fs::copy(
        "./tests/frontends/python/tokens.py",
        build.path().join("tokens.py"),
    )?;
    dbg!("hi");

    Command::new("python3")
        .arg("main.py")
        .current_dir(build.path())
        .run()?;

    Ok(())
}
