use anyhow::Context as Ctx;
use anyhow::Result;
use clap::Args;
use std::{fmt, fs, io, path};

#[derive(Args)]
pub struct MD5 {
    /// Files to digest (optional; default is stdin).
    /// With no FILE, or when FILE is -, read standard input.
    file: Option<Vec<path::PathBuf>>,
}

impl MD5 {
    pub fn exec(&self) -> Result<()> {
        macro_rules! print_hash {
            ( $name:expr, $no_buf_reader:expr ) => {
                let mut buf_r = io::BufReader::new($no_buf_reader);
                let mut hasher = Context::new();
                io::copy(&mut buf_r, &mut hasher).with_context(|| "could not read data")?;
                println!("{} {}", hasher.compute(), $name);
            };
        }
        match &self.file {
            Some(files) => {
                for file in files.iter() {
                    let name = file.to_str().unwrap_or("-");
                    if name == "-" {
                        print_hash!("-", io::stdin());
                        continue;
                    }
                    print_hash!(
                        name,
                        fs::File::open(file)
                            .with_context(|| format!("could not open file `{}`", name))?
                    );
                }
            }
            None => {
                print_hash!("-", io::stdin());
            }
        };
        Ok(())
    }
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
