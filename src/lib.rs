use anyhow::Result;
use clap::{Parser, Subcommand};

mod hash;
mod base64;
mod libs;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// compute and check MD5 message digest
    MD5(hash::Hash),
    /// compute and check SHA256 message digest
    SHA256(hash::Hash),
    Base64(base64::Base64)
}

impl Cli {
    pub fn new() -> Self {
        Cli::parse()
    }

    pub fn run(self) -> Result<()> {
        match self.command {
            Commands::MD5(cmd) => cmd.exec_md5()?,
            Commands::SHA256(cmd) => cmd.exec_sha256()?,
            Commands::Base64(cmd) => cmd.exec()?,
        }
        Ok(())
    }
}
