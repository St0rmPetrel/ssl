use anyhow::Result;

use std::io::Read;
use std::{fs, io, path::PathBuf};

pub enum Input<'a> {
    File(fs::File),
    Stdin(io::StdinLock<'a>),
}

impl<'a> Input<'a> {
    pub fn new(name: &PathBuf) -> Result<Input<'a>> {
        match name.to_str().unwrap() {
            "-" => Ok(Input::Stdin(io::stdin().lock())),
            _ => Ok(Input::File(fs::File::open(name)?)),
        }
    }
}

impl<'a> Read for Input<'a> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match *self {
            Input::File(ref mut file) => file.read(buf),
            Input::Stdin(ref mut stdin) => stdin.read(buf),
        }
    }
}
