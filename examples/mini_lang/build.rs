use std::{env, fs, path::PathBuf, process::Command};

fn main() {
    println!("cargo:rerun-if-changed=src/grammar.ast");
    println!("cargo:rerun-if-changed=build.rs");

    let mut output = PathBuf::from(env::var("OUT_DIR").unwrap());
    fs::create_dir_all(&output).expect("could not create OUT_DIR");
    output.push("parser.rs");
    let mut cmd = Command::new("asterisk-rs")
        .arg("src/grammar.ast")
        .arg("-o")
        .arg(output)
        .spawn()
        .expect("failed to spawn asterisk-rs");

    cmd.wait().expect("failed to wait for asterisk-rs");
}
