use clap::{Parser, Subcommand};
use std::error;
use std::fmt;

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

#[derive(Debug)]
pub enum Error {
    HashMD5(hash::Error),
    HashSHA256(hash::Error),
    Base64(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::HashMD5(err) => write!(f, "MD5: {}", err),
            Error::HashSHA256(err) => write!(f, "SHA256: {}", err),
            Error::Base64(err) => write!(f, "Base64: {}", err),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            Error::HashSHA256(ref e) => Some(e),
            Error::HashMD5(ref e) => Some(e),
            Error::Base64(_) => None,
        }
    }
}
