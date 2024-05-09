use std::io;

const INPUT_CHUNK_BYTE_SIZE: usize = 3;
const OUTPUT_CHUNK_BYTE_SIZE: usize = 4;
const PADDING: [u8; INPUT_CHUNK_BYTE_SIZE] = [0x00, 0x00, 0x00];
const CODE_VEC: [u8; 64] = [
    b'A', b'B', b'C', b'D', b'E', b'F', b'G', b'H', b'I', b'J', b'K', b'L', b'M', b'N', b'O', b'P',
    b'Q', b'R', b'S', b'T', b'U', b'V', b'W', b'X', b'Y', b'Z', b'a', b'b', b'c', b'd', b'e', b'f',
    b'g', b'h', b'i', b'j', b'k', b'l', b'm', b'n', b'o', b'p', b'q', b'r', b's', b't', b'u', b'v',
    b'w', b'x', b'y', b'z', b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'+', b'/',
];

pub struct Encoder<W: io::Write> {
    buf: [u8; INPUT_CHUNK_BYTE_SIZE],
    buf_seed: usize,
    encode_data: [u8; OUTPUT_CHUNK_BYTE_SIZE],
    writer: Option<W>,
}

impl<W: io::Write> io::Write for Encoder<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if self.writer.is_none() {
            panic!("Writer must be present");
        }

        let consume_bytes = self.write_buf(&buf);
        if self.is_buf_full() {
            self.encode();
            self.buf_seed = 0;

            let writer = self.writer.as_mut().unwrap();
            if let Err(err) = writer.write_all(&self.encode_data) {
                return Err(err);
            }
        }

        Ok(consume_bytes)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer
            .as_mut()
            .expect("Writer must be present")
            .flush()
    }
}

impl<W: io::Write> Drop for Encoder<W> {
    fn drop(&mut self) {
        let _ = self.finish();
    }
}

impl<W: io::Write> Encoder<W> {
    pub fn new(writer: W) -> Self {
        Encoder {
            buf: [0; INPUT_CHUNK_BYTE_SIZE],
            buf_seed: 0,
            encode_data: [0; OUTPUT_CHUNK_BYTE_SIZE],
            writer: Some(writer),
        }
    }

    pub fn finish(&mut self) -> io::Result<()> {
        if self.writer.is_none() {
            return Ok(());
        }

        let mut writer = self.writer.take().unwrap();

        if self.buf_seed != 0 {
            // zero buf free space
            let buf_free_size = INPUT_CHUNK_BYTE_SIZE - self.buf_seed;
            self.buf[self.buf_seed..].clone_from_slice(&PADDING[..buf_free_size]);

            self.encode();
            self.buf_seed = 0;
            for i in self.encode_data.len() - buf_free_size..self.encode_data.len() {
                self.encode_data[i] = b'=';
            }

            if let Err(err) = writer.write_all(&self.encode_data) {
                return Err(err);
            }
        }

        writer.flush()
    }

    fn write_buf(&mut self, input: &[u8]) -> usize {
        let buf_free_size = INPUT_CHUNK_BYTE_SIZE - self.buf_seed;
        if buf_free_size < input.len() {
            self.buf[self.buf_seed..].clone_from_slice(&input[..buf_free_size]);
            self.buf_seed = INPUT_CHUNK_BYTE_SIZE;

            buf_free_size
        } else {
            let new_buf_seed = self.buf_seed + input.len();
            self.buf[self.buf_seed..new_buf_seed].clone_from_slice(&input);
            self.buf_seed = new_buf_seed;

            input.len()
        }
    }

    fn encode(&mut self) {
        let idx_0 = (self.buf[0] & 0b1111_1100) >> 2;
        let idx_1 = ((self.buf[0] & 0b0000_0011) << 4) | ((self.buf[1] & 0b1111_0000) >> 4);
        let idx_2 = ((self.buf[1] & 0b0000_1111) << 2) | ((self.buf[2] & 0b1100_0000) >> 6);
        let idx_3 = self.buf[2] & 0b0011_1111;

        self.encode_data[0] = CODE_VEC[idx_0 as usize];
        self.encode_data[1] = CODE_VEC[idx_1 as usize];
        self.encode_data[2] = CODE_VEC[idx_2 as usize];
        self.encode_data[3] = CODE_VEC[idx_3 as usize];
    }

    fn is_buf_full(&self) -> bool {
        self.buf_seed == INPUT_CHUNK_BYTE_SIZE
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::vec::Vec;

    macro_rules! encoder {
        ($name:ident,$data:expr,$expected:expr) => {
            #[test]
            fn $name() {
                let mut out = Vec::new();
                {
                    let writer = io::BufWriter::new(&mut out);
                    let mut encoder = Encoder::new(writer);

                    write!(&mut encoder, $data).unwrap();
                }

                let actual = String::from_utf8(out).unwrap();

                println!("  actual: {:X?}", actual);
                println!("expected: {:X?}", $expected);

                assert_eq!($expected, actual);
            }
        };
    }

    encoder!(empty, "", "");
    encoder!(a, "a", "YQ==");
    encoder!(aa, "aa", "YWE=");
    encoder!(aaa, "aaa", "YWFh");
    encoder!(aaaa, "aaaa", "YWFhYQ==");
    encoder!(hello, "hello", "aGVsbG8=");
}
