use std::io;
use std::marker;

const NEW_LINE: [u8; 1] = [b'\n'];

pub struct NewLiner<W: io::Write + ?marker::Sized> {
    seed: usize,
    line_size: usize,
    writer: W,
}

impl<W: io::Write + ?marker::Sized> io::Write for NewLiner<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if self.seed == self.line_size {
            self.writer.write_all(&NEW_LINE)?;
            self.seed = 0;
        }

        let space = self.line_size - self.seed;
        let buf = if buf.len() < space {
            &buf
        } else {
            &buf[..space]
        };

        let writen = self.writer.write(&buf)?;
        self.seed += writen;
        Ok(writen)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

impl<W: io::Write> NewLiner<W> {
    pub fn with_line_size(line_size: usize, writer: W) -> Self {
        NewLiner {
            seed: 0,
            line_size,
            writer,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::vec::Vec;

    macro_rules! new_liner {
        ($name:ident,$line_size:expr,$data:expr,$expected:expr) => {
            #[test]
            fn $name() {
                let mut out = Vec::new();
                {
                    let writer = io::BufWriter::new(&mut out);
                    let mut new_liner = NewLiner::with_line_size($line_size, writer);

                    write!(&mut new_liner, $data).unwrap();
                }

                let actual = String::from_utf8(out).unwrap();

                println!("  actual: {:X?}", actual);
                println!("expected: {:X?}", $expected);

                assert_eq!($expected, actual);
            }
        };
    }

    new_liner!(empty, 1, "", "");
    new_liner!(a, 1, "a", "a");
    new_liner!(aa, 1, "aa", "a\na");
    new_liner!(aaa1, 1, "aaa", "a\na\na");
    new_liner!(aaa2, 2, "aaa", "aa\na");
    new_liner!(aaa3, 3, "aaa", "aaa");
}
