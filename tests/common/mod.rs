use std::{path::Path, process::Command};

use tempdir::TempDir;

pub fn asterisk() -> Command {
    let mut cmd = Command::new("cargo");
    cmd.arg("run").arg("-q").arg("--bin").arg("asterisk-rs").arg("--");
    cmd
}

pub fn asterisk_gen<P: AsRef<Path>, S: AsRef<Path>>(output: P, source: S) -> Command {
    let mut cmd = asterisk();
    cmd.arg("--format").arg("--output").arg(output.as_ref()).arg(source.as_ref());
    cmd
}

pub fn build_dir() -> TempDir {
    TempDir::new("asterisk").expect("could not create temporary dir")
}

pub trait CommandExt {
    fn run(&mut self) -> std::io::Result<()>;
}

impl CommandExt for Command {
    fn run(&mut self) -> std::io::Result<()> {
        let code = self.spawn()?.wait()?;
        assert!(code.success());
        Ok(())
    }
}
