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

pub fn as_u8_be(x: u32) -> [u8; 4] {
    let mut bytes = [0u8; 4];

    bytes[3] = (x & 0xff) as u8;
    bytes[2] = ((x >> 8) & 0xff) as u8;
    bytes[1] = ((x >> 16) & 0xff) as u8;
    bytes[0] = ((x >> 24) & 0xff) as u8;

    bytes
}

pub fn as_u32_be(bytes: &[u8]) -> u32 {
    ((bytes[0] as u32) << 24)
        + ((bytes[1] as u32) << 16)
        + ((bytes[2] as u32) << 8)
        + ((bytes[3] as u32) << 0)
}

pub fn right_rotate(x: u32, s: usize) -> u32 {
    (x >> s) | (x << (32 - s))
}

mod tests {
    use super::*;
    macro_rules! right_rotate_test {
        ($name:ident,$expected:expr,$data:expr,$shift:expr) => {
            #[test]
            fn $name() {
                let actual = right_rotate($data, $shift);

                println!("  actual: {:X?}", actual);
                println!("expected: {:X?}", $expected);
                assert_eq!($expected, actual);
            }
        };
    }

    right_rotate_test!(rr_byte_shift, 0xffff00ffu32, 0xff00ffffu32, 8);
    right_rotate_test!(rr_bits_1, 0xffff00ffu32, 0xff00ffffu32, 8);
}
