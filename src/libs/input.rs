use std::fs;
use std::io;
use std::path;

pub enum Input<'a> {
    File(fs::File),
    Stdin(io::StdinLock<'a>),
}

impl<'a> Input<'a> {
    pub fn new(file: &path::PathBuf) -> io::Result<Input<'a>> {
        match fs::File::open(file) {
            Ok(file) => Ok(Input::File(file)),
            Err(err) => match err.kind() {
                io::ErrorKind::NotFound => match file.to_str() {
                    Some("-") => Ok(Input::Stdin(io::stdin().lock())),
                    _ => Err(err),
                },
                _ => Err(err),
            },
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
