use anyhow::anyhow;
use anyhow::Context as Ctx;
use anyhow::Result;
use clap::Args;
use lazy_static::lazy_static;
use regex::Regex;

use std::io::{BufRead, Write};
use std::{fmt, io, path::PathBuf};

use crate::helper::Input;

#[derive(Args)]
pub struct MD5 {
    /// Files to digest (optional; default is stdin).
    /// With no FILE, or when FILE is -, read standard input.
    file: Option<Vec<PathBuf>>,

    /// create a BSD-style checksum if true.
    /// else create GNU style checksum file.
    #[arg(short, long)]
    tag: bool,
    /// read checksums from the FILEs and check them.
    #[arg(short, long)]
    check: bool,
}

impl MD5 {
    /// md5 command enter point.
    pub fn exec(self) -> Result<()> {
        if self.check {
            self.check()
        } else {
            self.create()
        }
    }

    /// read and check checksum file(s).
    /// compare for files listed in checksum file expected and actual computed hash of the file
    /// (among the list).
    fn check(self) -> Result<()> {
        let failed = self
            .file
            .unwrap_or(vec![PathBuf::from("-")])
            .into_iter()
            .map(|f| {
                let input = match Input::new(&f) {
                    Ok(input) => input,
                    Err(err) => {
                        eprintln!("{}", err);
                        return (f, 0, 1);
                    }
                };
                let failed = io::BufReader::new(input)
                    .lines()
                    .map(|l| check_line(&l?))
                    .filter_map(|x| x.err())
                    .fold(0, |acc, err| {
                        eprintln!("{}", err);
                        acc + 1
                    });
                (f, failed, 0)
            })
            .filter(|(_, check_failed, open_failed)| *check_failed > 0 || *open_failed > 0)
            .fold(0, |acc, (f, check_failed, open_failed)| {
                if check_failed > 0 {
                    eprintln!(
                        "WARNING: {} computed checksums did NOT match or FAIL to read: {}",
                        check_failed,
                        f.to_str().unwrap(),
                    );
                }
                if open_failed > 0 {
                    eprintln!("WARNING: FAIL to open: {}", f.to_str().unwrap(),);
                }
                acc + check_failed + open_failed
            });

        if failed > 0 {
            return Err(anyhow::anyhow!("WARNING: {} FAILS", failed));
        }
        Ok(())
    }

    /// create checksum file.
    fn create(self) -> Result<()> {
        // if no files in self.file add explicit stdin "-"
        let failed = self
            .file
            .unwrap_or(vec![PathBuf::from("-")])
            .into_iter()
            .map(|f| -> Result<()> {
                let digest = hash_file(&f)?;
                print_file(&f, digest, self.tag);
                Ok(())
            })
            .filter_map(|x| x.err())
            .fold(0, |acc, err| {
                eprintln!("{}", err);
                acc + 1
            });
        if failed > 0 {
            return Err(anyhow::anyhow!("WARNING: {} FAILS", failed));
        }
        Ok(())
    }
}

// print file digest in specific format.
fn print_file(file: &PathBuf, digest: Digest, is_bsd: bool) {
    let name = file.to_str().unwrap_or("-");
    if is_bsd {
        // BSD style checksum file
        println!("MD5 ({}) = {}", name, digest)
    } else {
        // GNU style checksum file
        println!("{}  {}", digest, name)
    }
}

/// read file (could be stdin "-") calculate hash of the file data
fn hash_file(file: &PathBuf) -> Result<Digest> {
    let mut buf_r =
        Input::new(&file).with_context(|| format!("fail to open {}", file.to_str().unwrap()))?;
    let mut hasher = Context::new();

    io::copy(&mut buf_r, &mut hasher)
        .with_context(|| format!("fail to read {}", file.to_str().unwrap()))?;

    Ok(hasher.compute())
}

/// parse checksum file line, line can be in GNU or BSD style format.
/// return filename and expected file hash.
fn parse_checksum_line(line: &str) -> Result<(PathBuf, Digest)> {
    lazy_static! {
        static ref GNU_STYLE_RE: Regex =
            Regex::new(r"^([[:alpha:]|0-9]{32})[[:space:]]+(.+)$").unwrap();
    }
    lazy_static! {
        static ref BSD_STYLE_RE: Regex =
            Regex::new(r"^MD5 \((.+)\)[[:space:]]*={1}[[:space:]]*([[:alpha:]|0-9]{32})$").unwrap();
    }

    if GNU_STYLE_RE.is_match(line) {
        let caps = GNU_STYLE_RE.captures(line).unwrap();
        let filename = PathBuf::from(caps.get(2).unwrap().as_str());
        let expected_digest = Digest::from_str(caps.get(1).unwrap().as_str())?;
        Ok((filename, expected_digest))
    } else if BSD_STYLE_RE.is_match(line) {
        let caps = BSD_STYLE_RE.captures(line).unwrap();
        let filename = PathBuf::from(caps.get(1).unwrap().as_str());
        let expected_digest = Digest::from_str(caps.get(2).unwrap().as_str())?;
        Ok((filename, expected_digest))
    } else {
        Err(anyhow::anyhow!("fail to parse line: {}", line))
    }
}

/// check line in checksum file
fn check_line(line: &str) -> Result<()> {
    let (file_name, expected_digest) = parse_checksum_line(line)?;

    let actual_digest = match hash_file(&file_name) {
        Ok(digest) => digest,
        Err(err) => {
            println!("{}: FAILED open or read", file_name.to_str().unwrap());
            return Err(err);
        }
    };

    let file_name = file_name.to_str().unwrap();

    if actual_digest != expected_digest {
        println!("{}: FAILED", file_name);
        return Err(anyhow!("computed checksum did NOT match: {}", file_name));
    } else {
        println!("{}: OK", file_name);
        Ok(())
    }
}

#[derive(Debug, PartialEq)]
pub struct Digest([u8; 16]);

impl fmt::Display for Digest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.0.iter() {
            let res = write!(f, "{:0>2x}", byte);
            if res.is_err() {
                return res;
            }
        }
        Ok(())
    }
}

impl Digest {
    fn from_state(a_s: u32, b_s: u32, c_s: u32, d_s: u32) -> Digest {
        let mut digest = [0u8; 16];
        digest[0..4].clone_from_slice(&as_u8_le(a_s));
        digest[4..8].clone_from_slice(&as_u8_le(b_s));
        digest[8..12].clone_from_slice(&as_u8_le(c_s));
        digest[12..16].clone_from_slice(&as_u8_le(d_s));

        Digest(digest)
    }

    fn from_str(s: &str) -> Result<Digest> {
        let mut digest = [0u8; 16];
        digest
            .iter_mut()
            .enumerate()
            .for_each(|(i, x)| *x = u8::from_str_radix(&s[2 * i..2 * i + 2], 16).unwrap());

        Ok(Digest(digest))
    }
}

const A0: u32 = 0x67452301;
const B0: u32 = 0xefcdab89;
const C0: u32 = 0x98badcfe;
const D0: u32 = 0x10325476;

const CHUNK_BYTE_SIZE: usize = 64;

pub struct Context {
    buf: [u8; CHUNK_BYTE_SIZE],
    buf_seed: usize,
    data_bytes_len: usize,

    a_s: u32,
    b_s: u32,
    c_s: u32,
    d_s: u32,
}

impl Write for Context {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.consume(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Context {
    /// Create new Context to md5 hash calculation, with initial values.
    pub fn new() -> Context {
        Context {
            buf: [0; CHUNK_BYTE_SIZE],
            buf_seed: 0,
            data_bytes_len: 0,

            a_s: A0,
            b_s: B0,
            c_s: C0,
            d_s: D0,
        }
    }
    /// Add last md5 chunks to data (padding and length of data), consume it and then
    /// return state (hash) of the Context.
    pub fn compute(self) -> Digest {
        let state = self.consume_final();

        Digest::from_state(state.0, state.1, state.2, state.3)
    }
}

const PADDING: [u8; 64] = [
    0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

const DATA_BITS_LENGTH_BYTE_SIZE: usize = 8;
const END_OF_DATA_BYTE_SIZE: usize = 1;

impl Context {
    /// Consume data, calculate new state for each md5 word (512 bits).
    fn consume(&mut self, mut data: &[u8]) {
        self.data_bytes_len = self.data_bytes_len.wrapping_add(data.len());

        while self.buf_seed + data.len() > CHUNK_BYTE_SIZE {
            self.buf[self.buf_seed..CHUNK_BYTE_SIZE]
                .clone_from_slice(&data[..CHUNK_BYTE_SIZE - self.buf_seed]);
            self.eat_chunk();
            data = &data[CHUNK_BYTE_SIZE - self.buf_seed..];
            self.buf_seed = 0;
        }
        self.buf[self.buf_seed..self.buf_seed + data.len()].clone_from_slice(data);
        self.buf_seed += data.len();
    }

    /// Create last chunk(s), and consume it(s).
    fn consume_final(mut self) -> (u32, u32, u32, u32) {
        let data_bits_len = (self.data_bytes_len as u64).wrapping_mul(8);
        // check self.buf_seed
        // if buf_seed > 64 - 9 => two final chunks
        // else => one final chunk
        if self.buf_seed <= CHUNK_BYTE_SIZE - (END_OF_DATA_BYTE_SIZE + DATA_BITS_LENGTH_BYTE_SIZE) {
            let pading_bytes_len = CHUNK_BYTE_SIZE - DATA_BITS_LENGTH_BYTE_SIZE - self.buf_seed;
            self.buf[self.buf_seed..self.buf_seed + pading_bytes_len]
                .clone_from_slice(&PADDING[..pading_bytes_len]);
            self.fill_data_len(data_bits_len);
            //println!("{:x?}", &self.buf);
            self.eat_chunk();
        } else {
            // chunk 1
            let pading_bytes_len = CHUNK_BYTE_SIZE - self.buf_seed;
            self.buf[self.buf_seed..self.buf_seed + pading_bytes_len]
                .clone_from_slice(&PADDING[..pading_bytes_len]);
            //println!("{:x?}", &self.buf);
            self.eat_chunk();

            // chunk 2
            self.buf[..CHUNK_BYTE_SIZE - DATA_BITS_LENGTH_BYTE_SIZE]
                .clone_from_slice(&PADDING[DATA_BITS_LENGTH_BYTE_SIZE..]);
            if pading_bytes_len == 0 {
                self.buf[0] = PADDING[0];
            }
            self.fill_data_len(data_bits_len);
            //println!("{:x?}", &self.buf);
            self.eat_chunk();
        }

        (self.a_s, self.b_s, self.c_s, self.d_s)
    }

    /// Fill length of data in bits in little endian format in the end of the last chunk.
    fn fill_data_len(&mut self, bits_len: u64) {
        self.buf[CHUNK_BYTE_SIZE - 8] = (bits_len & 0xff) as u8;
        self.buf[CHUNK_BYTE_SIZE - 7] = ((bits_len >> 8) & 0xff) as u8;
        self.buf[CHUNK_BYTE_SIZE - 6] = ((bits_len >> 16) & 0xff) as u8;
        self.buf[CHUNK_BYTE_SIZE - 5] = ((bits_len >> 24) & 0xff) as u8;
        self.buf[CHUNK_BYTE_SIZE - 4] = ((bits_len >> 32) & 0xff) as u8;
        self.buf[CHUNK_BYTE_SIZE - 3] = ((bits_len >> 40) & 0xff) as u8;
        self.buf[CHUNK_BYTE_SIZE - 2] = ((bits_len >> 48) & 0xff) as u8;
        self.buf[CHUNK_BYTE_SIZE - 1] = ((bits_len >> 56) & 0xff) as u8;
    }
}

const S: [usize; 64] = [
    7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 5, 9, 14, 20, 5, 9, 14, 20, 5, 9,
    14, 20, 5, 9, 14, 20, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 6, 10, 15,
    21, 6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21,
];

const K: [u32; 64] = [
    0xd76aa478, 0xe8c7b756, 0x242070db, 0xc1bdceee, 0xf57c0faf, 0x4787c62a, 0xa8304613, 0xfd469501,
    0x698098d8, 0x8b44f7af, 0xffff5bb1, 0x895cd7be, 0x6b901122, 0xfd987193, 0xa679438e, 0x49b40821,
    0xf61e2562, 0xc040b340, 0x265e5a51, 0xe9b6c7aa, 0xd62f105d, 0x02441453, 0xd8a1e681, 0xe7d3fbc8,
    0x21e1cde6, 0xc33707d6, 0xf4d50d87, 0x455a14ed, 0xa9e3e905, 0xfcefa3f8, 0x676f02d9, 0x8d2a4c8a,
    0xfffa3942, 0x8771f681, 0x6d9d6122, 0xfde5380c, 0xa4beea44, 0x4bdecfa9, 0xf6bb4b60, 0xbebfbc70,
    0x289b7ec6, 0xeaa127fa, 0xd4ef3085, 0x04881d05, 0xd9d4d039, 0xe6db99e5, 0x1fa27cf8, 0xc4ac5665,
    0xf4292244, 0x432aff97, 0xab9423a7, 0xfc93a039, 0x655b59c3, 0x8f0ccc92, 0xffeff47d, 0x85845dd1,
    0x6fa87e4f, 0xfe2ce6e0, 0xa3014314, 0x4e0811a1, 0xf7537e82, 0xbd3af235, 0x2ad7d2bb, 0xeb86d391,
];

impl Context {
    /// Consume whole chunk (512-bits), one state iteration.
    fn eat_chunk(&mut self) {
        let words = split_words(self.buf);

        let (mut a_temp, mut b_temp, mut c_temp, mut d_temp) =
            (self.a_s, self.b_s, self.c_s, self.d_s);

        let mut f_temp: u32 = 0;
        let mut g_temp: usize = 0;
        for i in 0usize..64 {
            if i < 16 {
                f_temp = (b_temp & c_temp) | ((!b_temp) & d_temp);
                g_temp = i;
            } else if i < 32 {
                f_temp = (d_temp & b_temp) | ((!d_temp) & c_temp);
                g_temp = (5 * i + 1) % 16;
            } else if i < 48 {
                f_temp = b_temp ^ c_temp ^ d_temp;
                g_temp = (3 * i + 5) % 16;
            } else if i < 64 {
                f_temp = c_temp ^ (b_temp | (!d_temp));
                g_temp = (7 * i) % 16;
            }

            f_temp = f_temp.wrapping_add(a_temp.wrapping_add(K[i]).wrapping_add(words[g_temp]));
            a_temp = d_temp;
            d_temp = c_temp;
            c_temp = b_temp;
            b_temp = b_temp.wrapping_add(left_rotate(f_temp, S[i]));
        }

        self.a_s = self.a_s.wrapping_add(a_temp);
        self.b_s = self.b_s.wrapping_add(b_temp);
        self.c_s = self.c_s.wrapping_add(c_temp);
        self.d_s = self.d_s.wrapping_add(d_temp);
    }
}

fn split_words(chunk: [u8; 64]) -> [u32; 16] {
    let mut words: [u32; 16] = [0; 16];

    for (i, word) in chunk.chunks(4).enumerate() {
        //     words[i] = as_u32_be(word);
        words[i] = as_u32_le(word);
    }

    words
}

fn left_rotate(x: u32, s: usize) -> u32 {
    (x << s) | (x >> (32 - s))
}

fn as_u32_le(bytes: &[u8]) -> u32 {
    ((bytes[0] as u32) << 0)
        + ((bytes[1] as u32) << 8)
        + ((bytes[2] as u32) << 16)
        + ((bytes[3] as u32) << 24)
}

fn as_u8_le(x: u32) -> [u8; 4] {
    let mut bytes = [0u8; 4];

    bytes[0] = (x & 0xff) as u8;
    bytes[1] = ((x >> 8) & 0xff) as u8;
    bytes[2] = ((x >> 16) & 0xff) as u8;
    bytes[3] = ((x >> 24) & 0xff) as u8;

    bytes
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! context {
        ($name:ident,$expected:expr,$data:expr) => {
            #[test]
            fn $name() {
                let mut ctx = Context::new();

                ctx.write(&$data).unwrap();

                let actual = ctx.compute().0;

                println!("actual: {:X?}", actual);
                println!("expected: {:X?}", $expected);

                assert_eq!($expected, actual);
            }
        };

        ($name:ident,$chunks:expr, $expected:expr,$data:expr) => {
            #[test]
            fn $name() {
                let mut ctx = Context::new();

                for chunk in $data.chunks($chunks) {
                    ctx.write(&chunk).unwrap();
                }

                let actual = ctx.compute().0;

                println!("actual: {:X?}", actual);
                println!("expected: {:X?}", $expected);

                assert_eq!($expected, actual);
            }
        };
    }

    context!(
        nothing,
        [
            0xd4, 0x1d, 0x8c, 0xd9, 0x8f, 0x0, 0xb2, 0x4, 0xe9, 0x80, 0x9, 0x98, 0xec, 0xf8, 0x42,
            0x7e
        ],
        // empty data
        []
    );
    context!(
        hello,
        [
            0xeb, 0x61, 0xee, 0xad, 0x90, 0xe3, 0xb8, 0x99, 0xc6, 0xbc, 0xbe, 0x27, 0xac, 0x58,
            0x16, 0x60
        ],
        // 'HELLO' ascii
        [0x48, 0x45, 0x4c, 0x4c, 0x4f]
    );
    context!(
        hello_chunk,
        3,
        [
            0xeb, 0x61, 0xee, 0xad, 0x90, 0xe3, 0xb8, 0x99, 0xc6, 0xbc, 0xbe, 0x27, 0xac, 0x58,
            0x16, 0x60
        ],
        // 'HELLO' ascii
        [0x48, 0x45, 0x4c, 0x4c, 0x4f]
    );
    context!(
        a_1000,
        [
            0x76, 0x44, 0x67, 0x2d, 0x04, 0x92, 0x90, 0xf0, 0x39, 0x0d, 0x9c, 0x99, 0x3c, 0x7d,
            0x34, 0x3d
        ],
        [0x41; 1000]
    );
    context!(
        a_1000_chunk,
        7,
        [
            0x76, 0x44, 0x67, 0x2d, 0x04, 0x92, 0x90, 0xf0, 0x39, 0x0d, 0x9c, 0x99, 0x3c, 0x7d,
            0x34, 0x3d
        ],
        [0x41; 1000]
    );
    context!(
        // double final chunk case
        a_1018,
        [
            0xb7, 0xdf, 0xfc, 0x69, 0x9b, 0x08, 0x1a, 0x6c, 0x9f, 0xd0, 0x59, 0x73, 0xf1, 0xd2,
            0x33, 0x60
        ],
        [0x41; 1018]
    );
    context!(
        // double final chunk case
        a_1018_chunk,
        17,
        [
            0xb7, 0xdf, 0xfc, 0x69, 0x9b, 0x08, 0x1a, 0x6c, 0x9f, 0xd0, 0x59, 0x73, 0xf1, 0xd2,
            0x33, 0x60
        ],
        [0x41; 1018]
    );

    context!(
        a_51,
        [
            0x8f, 0xe4, 0x66, 0x66, 0xaf, 0x29, 0x8b, 0xf2, 0xc1, 0x02, 0x2a, 0x62, 0x8d, 0x73,
            0xe9, 0x54
        ],
        [0x41; 51]
    );
    context!(
        a_64,
        [
            0xd2, 0x89, 0xa9, 0x75, 0x65, 0xbc, 0x2d, 0x27, 0xac, 0x8b, 0x85, 0x45, 0xa5, 0xdd,
            0xba, 0x45
        ],
        [0x41; 64]
    );
    context!(
        a_55,
        [
            0xe3, 0x8a, 0x93, 0xff, 0xe0, 0x74, 0xa9, 0x9b, 0x3f, 0xed, 0x47, 0xdf, 0xbe, 0x37,
            0xdb, 0x21
        ],
        [0x41; 55]
    );
}
