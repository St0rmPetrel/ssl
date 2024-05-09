use std::fs;
use std::io;
use std::path;

pub enum Input<'a> {
    File(fs::File),
    Stdin(io::StdinLock<'a>),
}

impl<'a> Input<'a> {
    pub fn new(name: &path::PathBuf) -> io::Result<Input<'a>> {
        match name.to_str().unwrap() {
            "-" => Ok(Input::Stdin(io::stdin().lock())),
            _ => Ok(Input::File(fs::File::open(name)?)),
        }
    }
}

impl<'a> io::Read for Input<'a> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match *self {
            Input::File(ref mut file) => file.read(buf),
            Input::Stdin(ref mut stdin) => stdin.read(buf),
        }
    }
}
