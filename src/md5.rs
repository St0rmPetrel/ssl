use anyhow::Result;

use std::fmt;

use crate::hasher;
use crate::helper::{as_u32_le, as_u8_le, left_rotate};

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

const A0: u32 = 0x67452301;
const B0: u32 = 0xefcdab89;
const C0: u32 = 0x98badcfe;
const D0: u32 = 0x10325476;

const CHUNK_BYTE_SIZE: usize = 64;
const DIGEST_BYTE_SIZE: usize = 16;

#[derive(Debug, PartialEq)]
pub struct Digest([u8; DIGEST_BYTE_SIZE]);

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
    pub fn from_state(a_s: u32, b_s: u32, c_s: u32, d_s: u32) -> Digest {
        let mut digest = [0u8; 16];
        digest[0..4].clone_from_slice(&as_u8_le(a_s));
        digest[4..8].clone_from_slice(&as_u8_le(b_s));
        digest[8..12].clone_from_slice(&as_u8_le(c_s));
        digest[12..16].clone_from_slice(&as_u8_le(d_s));

        Digest(digest)
    }

    pub fn from_str(s: &str) -> Result<Digest> {
        let mut digest = [0u8; 16];
        digest
            .iter_mut()
            .enumerate()
            .for_each(|(i, x)| *x = u8::from_str_radix(&s[2 * i..2 * i + 2], 16).unwrap());

        Ok(Digest(digest))
    }
}

pub struct Context {
    a_s: u32,
    b_s: u32,
    c_s: u32,
    d_s: u32,
}

impl Context {
    pub fn new() -> Context {
        Context {
            a_s: A0,
            b_s: B0,
            c_s: C0,
            d_s: D0,
        }
    }
}

impl hasher::Context for Context {
    type Digest = Digest;
    fn compress(&mut self, chunk: &[u8; CHUNK_BYTE_SIZE]) {
        let words = split_words(chunk);

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
    fn get_digest(self) -> Digest {
        Digest::from_state(self.a_s, self.b_s, self.c_s, self.d_s)
    }
}

fn split_words(chunk: &[u8; 64]) -> [u32; 16] {
    let mut words: [u32; 16] = [0; 16];

    for (i, word) in chunk.chunks(4).enumerate() {
        words[i] = as_u32_le(word);
    }

    words
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    macro_rules! context {
        ($name:ident,$expected:expr,$data:expr) => {
            #[test]
            fn $name() {
                let ctx = Context::new();
                let mut hasher = hasher::Writer::new(ctx, hasher::Endian::Little);

                hasher.write(&$data).unwrap();

                let actual = hasher.compute().0;

                println!("  actual: {:X?}", actual);
                println!("expected: {:X?}", $expected);

                assert_eq!($expected, actual);
            }
        };

        ($name:ident,$chunks:expr, $expected:expr,$data:expr) => {
            #[test]
            fn $name() {
                let ctx = Context::new();
                let mut hasher = hasher::Writer::new(ctx, hasher::Endian::Little);

                for chunk in $data.chunks($chunks) {
                    hasher.write(&chunk).unwrap();
                }

                let actual = hasher.compute().0;

                println!("  actual: {:X?}", actual);
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
    context!(
        a_65,
        [
            0x16, 0x2b, 0x6d, 0x6e, 0xb1, 0x7c, 0xd9, 0xda, 0x55, 0xf9, 0x5f, 0x8c, 0x73, 0xa3,
            0x2d, 0xd
        ],
        ['A' as u8; 65]
    );
}
