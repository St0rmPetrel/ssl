use clap::Args;
use std::{
    fmt,
    io::{self, Write},
    path::PathBuf,
};

#[derive(Args)]
pub struct MD5 {
    /// Files to digest (optional; default is stdin).
    /// With no FILE, or when FILE is -, read standard input.
    file: Option<Vec<PathBuf>>,
}

impl MD5 {
    pub fn exec(&self) {
        match &self.file {
            Some(files) => {
                for file in files.iter() {
                    process_file(file)
                }
            }
            None => process_file(&PathBuf::from("-")),
        };
    }
}

fn process_file(file: &PathBuf) {
    let data = read_file(file);
    let mut ctx = Context::new();
    let _ = ctx.write(&data);

    println!("{} {}", ctx.compute(), file.display())
}

fn read_file(_file: &PathBuf) -> Vec<u8> {
    unimplemented!()
}

pub struct Digest([u8; 16]);

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
    // Я тут понавляпывался в кучу ошибок, количество говнокода ниже превышает все разумные пределы. Запасись каплями для глаз
    // Буду благодарен если созвонимся и разберем потом
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut coefs_repr: [String; 4] = Default::default();
        for (i, coef_name) in "ABCD".chars().enumerate() {
            let s = self.0[4*i..4*(i+1)].iter().rev().map(|x| format!(" {:x}", x)).reduce(|s, x| s + &x);
            coefs_repr[i] = format!("{}:{:?}\n", coef_name, s)
        }
        write!(f, "{:?}", coefs_repr.iter().fold(String::new(), |s, x| s + &x))
    }
}

impl Context {
    /// Create new Context to md5 hash calculation, with initial values.
    pub fn new() -> Context {
        unimplemented!()
    }
    /// Add last md5 word to data (padding and length of data), consume it and then
    /// return state (hash) of the Context.
    pub fn compute(self) -> Digest {
        unimplemented!()
    }
}

impl Context {
    /// Consume data, calculate new state for each md5 word (512 bits).
    fn consume(&mut self, _data: &[u8]) {
        unimplemented!()
    }
}
