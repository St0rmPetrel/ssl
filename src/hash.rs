mod check;
mod digest;

use clap::Args;
use std::error;
use std::fmt;
use std::io::BufRead;
use std::{io, path::PathBuf};

pub use crate::libs::hash::Func;
use crate::libs::input;

type Result<T> = std::result::Result<T, Error>;

#[derive(Args)]
pub struct Hash {
    /// Files to digest (optional; default is stdin).
    /// With no FILE, or when FILE is -, read standard input.
    files: Option<Vec<PathBuf>>,

    /// create a BSD-style checksum if true.
    /// else create GNU style checksum file.
    #[arg(short, long)]
    tag: bool,
    /// read checksums from the FILEs and check them.
    #[arg(short, long)]
    check: bool,
}

impl Hash {
    pub fn exec(self, algo: Func) -> Result<()> {
        let files = self.files.unwrap_or(vec![PathBuf::from("-")]);
        let style = if self.tag {
            digest::Style::BSD
        } else {
            digest::Style::GNU
        };

        match self.check {
            true => check(files),
            _ => digest(files, algo, style),
        }
    }
}

#[derive(Debug)]
pub struct Error {
    failed: usize,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "WARNING: {} FAILS", self.failed)
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}

/// read and check checksum file(s).
/// compare for files listed in checksum file expected and actual computed hash of the file
/// (among the list).
fn check(files: Vec<PathBuf>) -> Result<()> {
    let mut failed: usize = 0;
    for file in files.iter() {
        let r = match input::Input::new(&file) {
            Ok(input) => input,
            Err(err) => {
                eprintln!("{}", err);
                continue;
            }
        };

        let buf_r = io::BufReader::new(r);
        for line in buf_r.lines() {
            let line = match line {
                Ok(line) => line,
                Err(err) => {
                    eprintln!("read line: {}", err);
                    failed += 1;
                    continue;
                }
            };
            match check::line(&line) {
                // TODO: not file path in line.
                Ok(_) => println!("{:?} OK", file),
                Err(err) => {
                    eprintln!("check_line: file {:?}, line {:?}: {}", file,line, err);
                    failed += 1;
                    continue;
                }
            }
        }
    }

    if failed > 0 {
        Err(Error { failed })
    } else {
        Ok(())
    }
}

/// create checksum file.
fn digest(files: Vec<PathBuf>, algo: Func, style: digest::Style) -> Result<()> {
    let mut failed: usize = 0;
    for file in files.iter() {
        match digest::println(&file, algo, style) {
            Ok(_) => (),
            Err(err) => {
                eprintln!("digest {:?}: {}", file, err);
                failed += 1;
                continue;
            }
        };
    }

    if failed > 0 {
        Err(Error { failed })
    } else {
        Ok(())
    }
}
