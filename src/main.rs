use anyhow::Result;
use clap::{Parser, Subcommand};

mod hasher;
mod helper;
mod md5;
mod sha256;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// compute and check MD5 message digest
    MD5(md5::MD5),
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::MD5(cmd) => cmd.exec()?,
    }
    Ok(())
}
