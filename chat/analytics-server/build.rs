use anyhow::Result;
use std::{fs, process::Command};

fn main() -> Result<()> {
    fs::create_dir_all("src/pb")?;
    prost_build::Config::new()
        .out_dir("src/pb")
        .compile_protos(&["../../protos/messages.proto"], &["../../protos"])?;

    println!("cargo:rerun-if-changed=../../protos/messages.proto");
    // run fmt
    Command::new("cargo").arg("fmt").status()?;
    Ok(())
}
