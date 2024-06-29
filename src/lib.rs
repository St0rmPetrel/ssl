use clap::{Parser, Subcommand};
use std::error;

mod base64;
mod hash;
mod libs;

type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

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
    Base64(base64::Base64),
}

impl Cli {
    pub fn new() -> Self {
        Cli::parse()
    }

    pub fn run(self) -> Result<()> {
        match self.command {
            Commands::MD5(cmd) => cmd.exec(hash::Func::MD5)?,
            Commands::SHA256(cmd) => cmd.exec(hash::Func::SHA256)?,
            Commands::Base64(cmd) => cmd.exec()?,
        }
        Ok(())
    }
}
