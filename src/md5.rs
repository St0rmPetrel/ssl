use anyhow::Context as Ctx;
use anyhow::Result;
use clap::Args;
use std::{fmt, fs, io, path::PathBuf};

#[derive(Args)]
pub struct MD5 {
    /// Files to digest (optional; default is stdin).
    /// With no FILE, or when FILE is -, read standard input.
    file: Option<Vec<PathBuf>>,
}

impl MD5 {
    pub fn exec(self) -> Result<()> {
        // if no files in self.file add explicit stdin "-"
        for file in self.file.unwrap_or(vec![PathBuf::from("-")]) {
            let (name, digest) = hash_file(file)?;

            println!("{} {}", name, digest);
        }
        Ok(())
    }
}

/// read file (could be stdin "-") calculate hash of the file data
fn hash_file(file: PathBuf) -> Result<(String, Digest)> {
    let name = String::from(file.to_str().unwrap_or("-"));
    let mut buf_r: Box<dyn io::BufRead> = match name.as_str() {
        "-" => Box::new(io::BufReader::new(io::stdin())),
        _ => Box::new(io::BufReader::new(
            fs::File::open(file).with_context(|| format!("could not open file `{}`", name))?,
        )),
    };
    let mut hasher = Context::new();
    io::copy(&mut buf_r, &mut hasher).with_context(|| "could not read data")?;

    Ok((name, hasher.compute()))
}

pub struct Digest(pub [u8; 16]);

pub struct Context;

impl io::Write for Context {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.consume(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl fmt::Display for Digest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: implement me
        write!(f, "implement_me")
    }
}

impl Context {
    /// Create new Context to md5 hash calculation, with initial values.
    pub fn new() -> Context {
        // TODO: implement me
        Context {}
    }
    /// Add last md5 word to data (padding and length of data), consume it and then
    /// return state (hash) of the Context.
    pub fn compute(self) -> Digest {
        // TODO: implement me
        Digest([0; 16])
    }
}

impl Context {
    /// Consume data, calculate new state for each md5 word (512 bits).
    fn consume(&mut self, _data: &[u8]) {
        // TODO: implement me
    }
}
