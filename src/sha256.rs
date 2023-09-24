use anyhow::Result;

use std::fmt;

use crate::hasher;
use crate::helper::{as_u32_be, as_u8_be, right_rotate};

const DIGEST_WORD_SIZE: usize = 8;
const BYTES_IN_WORD: usize = 4;
const DIGEST_BYTE_SIZE: usize = DIGEST_WORD_SIZE * BYTES_IN_WORD;
const CHUNK_BYTE_SIZE: usize = 64;

const K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

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
    pub fn from_str(s: &str) -> Result<Digest> {
        let mut digest = [0u8; DIGEST_BYTE_SIZE];
        digest
            .iter_mut()
            .enumerate()
            .for_each(|(i, x)| *x = u8::from_str_radix(&s[2 * i..2 * i + 2], 16).unwrap());

        Ok(Digest(digest))
    }
}

pub struct Context {
    state: [u32; DIGEST_WORD_SIZE],
}

impl Context {
    pub fn new() -> Context {
        Context {
            state: [
                0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
                0x5be0cd19,
            ],
        }
    }
}

impl hasher::Context for Context {
    type Digest = Digest;

    fn compress(&mut self, chunk: &[u8; CHUNK_BYTE_SIZE]) {
        let (mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h) = (
            self.state[0],
            self.state[1],
            self.state[2],
            self.state[3],
            self.state[4],
            self.state[5],
            self.state[6],
            self.state[7],
        );
        let words = get_words(chunk);

        for i in 0..64 {
            let s1 = right_rotate(e, 6) ^ right_rotate(e, 11) ^ right_rotate(e, 25);
            let ch = (e & f) ^ ((!e) & g);
            let temp1 = h.wrapping_add(
                s1.wrapping_add(ch)
                    .wrapping_add(K[i])
                    .wrapping_add(words[i]),
            );

            let s0 = right_rotate(a, 2) ^ right_rotate(a, 13) ^ right_rotate(a, 22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);

            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }
        self.state[0] = a.wrapping_add(self.state[0]);
        self.state[1] = b.wrapping_add(self.state[1]);
        self.state[2] = c.wrapping_add(self.state[2]);
        self.state[3] = d.wrapping_add(self.state[3]);
        self.state[4] = e.wrapping_add(self.state[4]);
        self.state[5] = f.wrapping_add(self.state[5]);
        self.state[6] = g.wrapping_add(self.state[6]);
        self.state[7] = h.wrapping_add(self.state[7]);
    }

    fn get_digest(self) -> Digest {
        let mut digest = [0u8; DIGEST_BYTE_SIZE];
        for i in 0..DIGEST_WORD_SIZE {
            digest[i * 4..(i + 1) * 4].clone_from_slice(&as_u8_be(self.state[i]));
        }
        Digest(digest)
    }
}

fn get_words(chunk: &[u8; CHUNK_BYTE_SIZE]) -> [u32; 64] {
    let mut words: [u32; 64] = [0; 64];
    for (i, word) in chunk.chunks(BYTES_IN_WORD).enumerate() {
        words[i] = as_u32_be(word);
    }

    for i in 16..64 {
        let s0 =
            right_rotate(words[i - 15], 7) ^ right_rotate(words[i - 15], 18) ^ (words[i - 15] >> 3);
        let s1 =
            right_rotate(words[i - 2], 17) ^ right_rotate(words[i - 2], 19) ^ (words[i - 2] >> 10);
        words[i] = words[i - 16].wrapping_add(s0.wrapping_add(words[i - 7]).wrapping_add(s1));
    }

    words
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    macro_rules! ctx_test {
        ($name:ident,$expected:expr,$data:expr) => {
            #[test]
            fn $name() {
                let ctx = Context::new();
                let mut hasher = hasher::Writer::new(ctx, hasher::Endian::Big);

                hasher.write(&$data).unwrap();

                let actual = hasher.compute().0;

                println!("  actual: {:X?}", actual);
                println!("expected: {:X?}", $expected);

                assert_eq!($expected, actual);
            }
        };
    }

    ctx_test!(
        nothing,
        [
            0xe3, 0xb0, 0xc4, 0x42, 0x98, 0xfc, 0x1c, 0x14, 0x9a, 0xfb, 0xf4, 0xc8, 0x99, 0x6f,
            0xb9, 0x24, 0x27, 0xae, 0x41, 0xe4, 0x64, 0x9b, 0x93, 0x4c, 0xa4, 0x95, 0x99, 0x1b,
            0x78, 0x52, 0xb8, 0x55,
        ],
        // empty data
        []
    );
    ctx_test!(
        abc,
        [
            0xba, 0x78, 0x16, 0xbf, 0x8f, 0x01, 0xcf, 0xea, 0x41, 0x41, 0x40, 0xde, 0x5d, 0xae,
            0x22, 0x23, 0xb0, 0x03, 0x61, 0xa3, 0x96, 0x17, 0x7a, 0x9c, 0xb4, 0x10, 0xff, 0x61,
            0xf2, 0x00, 0x15, 0xad
        ],
        ['a' as u8, 'b' as u8, 'c' as u8]
    );
    ctx_test!(
        a_51,
        [
            0x1d, 0x31, 0x61, 0x6e, 0x30, 0x73, 0x23, 0xbd, 0x80, 0x77, 0x5a, 0xe7, 0x48, 0x3f,
            0xce, 0x65, 0x4a, 0x3b, 0x65, 0xbc, 0xed, 0x71, 0x34, 0xc2, 0x2e, 0x17, 0x9a, 0x2e,
            0x25, 0x15, 0x50, 0x09
        ],
        ['A' as u8; 51]
    );
    ctx_test!(
        a_64,
        [
            0xd5, 0x3e, 0xda, 0x7a, 0x63, 0x7c, 0x99, 0xcc, 0x7f, 0xb5, 0x66, 0xd9, 0x6e, 0x9f,
            0xa1, 0x09, 0xbf, 0x15, 0xc4, 0x78, 0x41, 0x0a, 0x3f, 0x5e, 0xb4, 0xd4, 0xc4, 0xe2,
            0x6c, 0xd0, 0x81, 0xf6
        ],
        ['A' as u8; 64]
    );
    ctx_test!(
        a_55,
        [
            0x89, 0x63, 0xcc, 0x0a, 0xfd, 0x62, 0x2c, 0xc7, 0x57, 0x4a, 0xc2, 0x01, 0x1f, 0x93,
            0xa3, 0x05, 0x9b, 0x3d, 0x65, 0x54, 0x8a, 0x77, 0x54, 0x2a, 0x15, 0x59, 0xe3, 0xd2,
            0x02, 0xe6, 0xab, 0x00
        ],
        ['A' as u8; 55]
    );
    ctx_test!(
        a_1000,
        [
            0xc2, 0xe6, 0x86, 0x82, 0x34, 0x89, 0xce, 0xd2, 0x01, 0x7f, 0x60, 0x59, 0xb8, 0xb2,
            0x39, 0x31, 0x8b, 0x63, 0x64, 0xf6, 0xdc, 0xd8, 0x35, 0xd0, 0xa5, 0x19, 0x10, 0x5a,
            0x1e, 0xad, 0xd6, 0xe4
        ],
        ['A' as u8; 1000]
    );
}
