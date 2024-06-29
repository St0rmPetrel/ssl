use lazy_static::lazy_static;
use regex::Regex;
use std::error;
use std::fmt;
use std::io;
use std::path::PathBuf;

use crate::libs::hash;
use crate::libs::hash::md5;
use crate::libs::hash::sha256;
use crate::libs::input;

#[derive(Debug)]
pub enum Error {
    DigestIncorrect,
    ParseChecksumLine(ParseChecksumLineError),
    Digest(io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::DigestIncorrect => write!(f, "digest incorrect"),
            Error::ParseChecksumLine(err) => write!(f, "parse checksumline: {}", err),
            Error::Digest(err) => write!(f, "digest: {}", err),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            Error::DigestIncorrect => None,
            Error::ParseChecksumLine(ref e) => Some(e),
            Error::Digest(ref e) => Some(e),
        }
    }
}

impl From<ParseChecksumLineError> for Error {
    fn from(err: ParseChecksumLineError) -> Error {
        Error::ParseChecksumLine(err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Digest(err)
    }
}

/// check line in checksum file
pub fn line(line: &str) -> Result<(), Error> {
    let (path, expected_digest) = parse_checksum_line(line)?;
    let r = input::Input::new(&path)?;

    let actual_digest = match expected_digest {
        hash::Digest::MD5(_) => hash::digest(r, hash::Func::MD5)?,
        hash::Digest::SHA256(_) => hash::digest(r, hash::Func::SHA256)?,
    };

    if expected_digest != actual_digest {
        Err(Error::DigestIncorrect)
    } else {
        Ok(())
    }
}

#[derive(Debug)]
pub enum ParseChecksumLineError {
    UnrecognizeLine,
    CapturePath,
    CaptureDigest,
    ParseDigest(ParseDigestError),
}

impl fmt::Display for ParseChecksumLineError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseChecksumLineError::UnrecognizeLine => write!(f, "line is unrecognize"),
            ParseChecksumLineError::CapturePath => write!(f, "fail to capture path"),
            ParseChecksumLineError::CaptureDigest => write!(f, "fail to capture digest"),
            ParseChecksumLineError::ParseDigest(err) => write!(f, "parse digest: {}", err),
        }
    }
}

impl error::Error for ParseChecksumLineError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            ParseChecksumLineError::UnrecognizeLine => None,
            ParseChecksumLineError::CapturePath => None,
            ParseChecksumLineError::CaptureDigest => None,
            ParseChecksumLineError::ParseDigest(ref e) => Some(e),
        }
    }
}

impl From<ParseDigestError> for ParseChecksumLineError {
    fn from(err: ParseDigestError) -> ParseChecksumLineError {
        ParseChecksumLineError::ParseDigest(err)
    }
}

fn parse_checksum_line(line: &str) -> Result<(PathBuf, hash::Digest), ParseChecksumLineError> {
    lazy_static! {
        static ref SHA256_GNU_STYLE_RE: Regex =
            Regex::new(r"^([[:alpha:]|0-9]{64})[[:space:]]+(.+)$")
                .expect("sha256 gnu regex must be valid");
    }
    lazy_static! {
        static ref SHA256_BSD_STYLE_RE: Regex =
            Regex::new(r"^SHA256 \((.+)\)[[:space:]]*={1}[[:space:]]*([[:alpha:]|0-9]{64})$")
                .expect("sha256 bsd regex must be valid");
    }
    lazy_static! {
        static ref MD5_GNU_STYLE_RE: Regex = Regex::new(r"^([[:alpha:]|0-9]{32})[[:space:]]+(.+)$")
            .expect("md5 gnu regex must be valid");
    }
    lazy_static! {
        static ref MD5_BSD_STYLE_RE: Regex =
            Regex::new(r"^MD5 \((.+)\)[[:space:]]*={1}[[:space:]]*([[:alpha:]|0-9]{32})$")
                .expect("md5 bsd regex must be valid");
    }

    let (path, expected_digest, hf) = if let Some(caps) = SHA256_GNU_STYLE_RE.captures(line) {
        let path = caps
            .get(2)
            .ok_or(ParseChecksumLineError::CapturePath)?
            .as_str();
        let expected_digest = caps
            .get(1)
            .ok_or(ParseChecksumLineError::CaptureDigest)?
            .as_str();
        (path, expected_digest, hash::Func::SHA256)
    } else if let Some(caps) = SHA256_BSD_STYLE_RE.captures(line) {
        let path = caps
            .get(1)
            .ok_or(ParseChecksumLineError::CapturePath)?
            .as_str();
        let expected_digest = caps
            .get(2)
            .ok_or(ParseChecksumLineError::CaptureDigest)?
            .as_str();
        (path, expected_digest, hash::Func::SHA256)
    } else if let Some(caps) = MD5_GNU_STYLE_RE.captures(line) {
        let path = caps
            .get(2)
            .ok_or(ParseChecksumLineError::CapturePath)?
            .as_str();
        let expected_digest = caps
            .get(1)
            .ok_or(ParseChecksumLineError::CaptureDigest)?
            .as_str();
        (path, expected_digest, hash::Func::SHA256)
    } else if let Some(caps) = MD5_GNU_STYLE_RE.captures(line) {
        let path = caps
            .get(1)
            .ok_or(ParseChecksumLineError::CapturePath)?
            .as_str();
        let expected_digest = caps
            .get(2)
            .ok_or(ParseChecksumLineError::CaptureDigest)?
            .as_str();
        (path, expected_digest, hash::Func::SHA256)
    } else {
        return Err(ParseChecksumLineError::UnrecognizeLine);
    };

    let path = PathBuf::from(path);
    let expected_digest = parse_digest(expected_digest, hf)?;

    Ok((path, expected_digest))
}

#[derive(Debug)]
pub enum ParseDigestError {
    InvalidStrLen { expected: usize, actual: usize },
    ParseByte(std::num::ParseIntError),
}

impl fmt::Display for ParseDigestError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseDigestError::InvalidStrLen { expected, actual } => write!(
                f,
                "invalid str length: expected {}, actual {}",
                expected, actual
            ),
            ParseDigestError::ParseByte(err) => write!(f, "parse byte: {}", err),
        }
    }
}

impl error::Error for ParseDigestError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            ParseDigestError::InvalidStrLen { .. } => None,
            ParseDigestError::ParseByte(ref e) => Some(e),
        }
    }
}

impl From<std::num::ParseIntError> for ParseDigestError {
    fn from(err: std::num::ParseIntError) -> ParseDigestError {
        ParseDigestError::ParseByte(err)
    }
}

fn parse_digest(s: &str, hf: hash::Func) -> Result<hash::Digest, ParseDigestError> {
    match hf {
        hash::Func::MD5 => Ok(hash::Digest::MD5(parse_digest_md5(s)?)),
        hash::Func::SHA256 => Ok(hash::Digest::SHA256(parse_digest_sha256(s)?)),
    }
}

fn parse_digest_md5(s: &str) -> Result<md5::Digest, ParseDigestError> {
    if s.len() != md5::DIGEST_STR_LEN {
        return Err(ParseDigestError::InvalidStrLen {
            expected: md5::DIGEST_STR_LEN,
            actual: s.len(),
        }
        .into());
    }

    let mut digest = [0u8; md5::DIGEST_BYTE_SIZE];

    for (i, x) in digest.iter_mut().enumerate() {
        *x = u8::from_str_radix(&s[2 * i..2 * i + 2], 16)?;
    }

    Ok(md5::Digest::new(digest))
}

fn parse_digest_sha256(s: &str) -> std::result::Result<sha256::Digest, ParseDigestError> {
    if s.len() != sha256::DIGEST_STR_LEN {
        return Err(ParseDigestError::InvalidStrLen {
            expected: sha256::DIGEST_STR_LEN,
            actual: s.len(),
        });
    }

    let mut digest = [0u8; sha256::DIGEST_BYTE_SIZE];

    for (i, x) in digest.iter_mut().enumerate() {
        *x = u8::from_str_radix(&s[2 * i..2 * i + 2], 16)?;
    }

    Ok(sha256::Digest::new(digest))
}
