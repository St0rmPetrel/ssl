pub mod md5;
pub mod sha256;

use std::fmt;
use std::io::{self, Write};

const CHUNK_BYTE_SIZE: usize = 64;
const PADDING: [u8; CHUNK_BYTE_SIZE] = [
    0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

const DATA_BITS_LENGTH_BYTE_SIZE: usize = 8;
const END_OF_DATA_BYTE_SIZE: usize = 1;

pub trait Context {
    type Digest;

    fn compress(&mut self, chunk: &[u8; CHUNK_BYTE_SIZE]);
    fn get_digest(self) -> Self::Digest;
}

#[derive(Debug)]
pub enum Endian {
    Big,
    Little,
}

#[derive(Debug, Clone, Copy)]
pub enum Func {
    MD5,
    SHA256,
}

impl fmt::Display for Func {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Func::MD5 => write!(f, "MD5"),
            Func::SHA256 => write!(f, "SHA256"),
        }
    }
}

#[derive(PartialEq)]
pub enum Digest {
    MD5(md5::Digest),
    SHA256(sha256::Digest),
}

impl fmt::Display for Digest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Digest::MD5(digest) => write!(f, "{}", digest),
            Digest::SHA256(digest) => write!(f, "{}", digest),
        }
    }
}

pub struct Writer<Ctx: Context> {
    buf: [u8; CHUNK_BYTE_SIZE],
    buf_seed: usize,
    data_bytes_len: usize,
    endian: Endian,
    hasher: Ctx,
}

pub fn digest<R: io::Read>(r: R, f: Func) -> io::Result<Digest> {
    match f {
        Func::MD5 => Ok(Digest::MD5(md5(r)?)),
        Func::SHA256 => Ok(Digest::SHA256(sha256(r)?)),
    }
}

pub fn md5<R: io::Read>(mut r: R) -> io::Result<md5::Digest> {
    let ctx = md5::Context::new();
    let mut hasher = Writer::new(ctx, Endian::Little);
    io::copy(&mut r, &mut hasher)?;

    Ok(hasher.compute())
}

pub fn sha256<R: io::Read>(mut r: R) -> io::Result<sha256::Digest> {
    let ctx = sha256::Context::new();
    let mut hasher = Writer::new(ctx, Endian::Big);
    io::copy(&mut r, &mut hasher)?;

    Ok(hasher.compute())
}

impl<Ctx: Context> Write for Writer<Ctx> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.consume(buf);

        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl<Ctx: Context> Writer<Ctx> {
    pub fn new(hasher: Ctx, endian: Endian) -> Writer<Ctx> {
        Writer {
            buf: [0; CHUNK_BYTE_SIZE],
            buf_seed: 0,
            data_bytes_len: 0,
            hasher,
            endian,
        }
    }

    pub fn compute(mut self) -> Ctx::Digest {
        let data_bits_len = (self.data_bytes_len as u64).wrapping_mul(8);
        // check self.buf_seed
        // if buf_seed > 64 - 9 => two final chunks
        // else => one final chunk
        if self.buf_seed <= CHUNK_BYTE_SIZE - (END_OF_DATA_BYTE_SIZE + DATA_BITS_LENGTH_BYTE_SIZE) {
            let pading_bytes_len = CHUNK_BYTE_SIZE - DATA_BITS_LENGTH_BYTE_SIZE - self.buf_seed;
            self.buf[self.buf_seed..self.buf_seed + pading_bytes_len]
                .clone_from_slice(&PADDING[..pading_bytes_len]);
            self.fill_data_len(data_bits_len);
            self.hasher.compress(&self.buf);
        } else {
            // chunk 1
            let pading_bytes_len = CHUNK_BYTE_SIZE - self.buf_seed;
            self.buf[self.buf_seed..self.buf_seed + pading_bytes_len]
                .clone_from_slice(&PADDING[..pading_bytes_len]);
            self.hasher.compress(&self.buf);

            // chunk 2
            self.buf[..CHUNK_BYTE_SIZE - DATA_BITS_LENGTH_BYTE_SIZE]
                .clone_from_slice(&PADDING[DATA_BITS_LENGTH_BYTE_SIZE..]);
            if pading_bytes_len == 0 {
                self.buf[0] = PADDING[0];
            }
            self.fill_data_len(data_bits_len);
            self.hasher.compress(&self.buf);
        }

        self.hasher.get_digest()
    }

    fn fill_data_len(&mut self, bits_len: u64) {
        match self.endian {
            Endian::Big => {
                self.buf[CHUNK_BYTE_SIZE - 1] = (bits_len & 0xff) as u8;
                self.buf[CHUNK_BYTE_SIZE - 2] = ((bits_len >> 8) & 0xff) as u8;
                self.buf[CHUNK_BYTE_SIZE - 3] = ((bits_len >> 16) & 0xff) as u8;
                self.buf[CHUNK_BYTE_SIZE - 4] = ((bits_len >> 24) & 0xff) as u8;
                self.buf[CHUNK_BYTE_SIZE - 5] = ((bits_len >> 32) & 0xff) as u8;
                self.buf[CHUNK_BYTE_SIZE - 6] = ((bits_len >> 40) & 0xff) as u8;
                self.buf[CHUNK_BYTE_SIZE - 7] = ((bits_len >> 48) & 0xff) as u8;
                self.buf[CHUNK_BYTE_SIZE - 8] = ((bits_len >> 56) & 0xff) as u8;
            }
            Endian::Little => {
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
    }

    fn consume(&mut self, mut buf: &[u8]) {
        self.data_bytes_len = self.data_bytes_len.wrapping_add(buf.len());

        while self.buf_seed + buf.len() > CHUNK_BYTE_SIZE {
            self.buf[self.buf_seed..CHUNK_BYTE_SIZE]
                .clone_from_slice(&buf[..CHUNK_BYTE_SIZE - self.buf_seed]);
            self.hasher.compress(&self.buf);
            buf = &buf[CHUNK_BYTE_SIZE - self.buf_seed..];
            self.buf_seed = 0;
        }
        self.buf[self.buf_seed..self.buf_seed + buf.len()].clone_from_slice(buf);
        self.buf_seed += buf.len();
    }
}
