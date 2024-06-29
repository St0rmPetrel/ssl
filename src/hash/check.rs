use lazy_static::lazy_static;
use regex::Regex;
use std::error;
use std::fmt;
use std::path::PathBuf;

use crate::libs::hash;
use crate::libs::hash::md5;
use crate::libs::hash::sha256;
use crate::libs::input;

/// check line in checksum file
pub fn line(line: &str, algo: &hash::Func) -> Result<(), Box<dyn error::Error>> {
    match algo {
        hash::Func::MD5 => {
            let (file_name, expected_digest) = parse_md5_checksum_line(line)?;
            let r = input::Input::new(&file_name)?;

            let actual_digest = match hash::md5(r) {
                Ok(digest) => digest,
                Err(err) => {
                    println!("{}: FAILED open or read", file_name.to_str().unwrap());
                    return Err(err.into());
                }
            };

            let file_name = file_name.to_str().unwrap();

            if actual_digest != expected_digest {
                println!("{}: FAILED", file_name);
                return Err(format!("computed checksum did NOT match: {}", file_name).into());
            } else {
                println!("{}: OK", file_name);
                Ok(())
            }
        }
        hash::Func::SHA256 => {
            let (file_name, expected_digest) = parse_sha256_checksum_line(line)?;
            let r = input::Input::new(&file_name)?;

            let actual_digest = match hash::sha256(r) {
                Ok(digest) => digest,
                Err(err) => {
                    println!("{}: FAILED open or read", file_name.to_str().unwrap());
                    return Err(err.into());
                }
            };

            let file_name = file_name.to_str().unwrap();

            if actual_digest != expected_digest {
                println!("{}: FAILED", file_name);
                return Err(format!("computed checksum did NOT match: {}", file_name).into());
            } else {
                println!("{}: OK", file_name);
                Ok(())
            }
        }
    }
}

fn parse_md5_checksum_line(line: &str) -> Result<(PathBuf, md5::Digest), Box<dyn error::Error>> {
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
        let expected_digest = parse_digest_md5(caps.get(1).unwrap().as_str())?;
        Ok((filename, expected_digest))
    } else if BSD_STYLE_RE.is_match(line) {
        let caps = BSD_STYLE_RE.captures(line).unwrap();
        let filename = PathBuf::from(caps.get(1).unwrap().as_str());
        let expected_digest = parse_digest_md5(caps.get(2).unwrap().as_str())?;
        Ok((filename, expected_digest))
    } else {
        Err(format!("fail to parse line: {}", line).into())
    }
}

fn parse_sha256_checksum_line(
    line: &str,
) -> Result<(PathBuf, sha256::Digest), Box<dyn error::Error>> {
    lazy_static! {
        static ref GNU_STYLE_RE: Regex =
            Regex::new(r"^([[:alpha:]|0-9]{64})[[:space:]]+(.+)$").unwrap();
    }
    lazy_static! {
        static ref BSD_STYLE_RE: Regex =
            Regex::new(r"^SHA256 \((.+)\)[[:space:]]*={1}[[:space:]]*([[:alpha:]|0-9]{64})$")
                .unwrap();
    }

    if GNU_STYLE_RE.is_match(line) {
        let caps = GNU_STYLE_RE.captures(line).unwrap();
        let filename = PathBuf::from(caps.get(2).unwrap().as_str());
        let expected_digest = parse_digest_sha256(caps.get(1).unwrap().as_str())?;
        Ok((filename, expected_digest))
    } else if BSD_STYLE_RE.is_match(line) {
        let caps = BSD_STYLE_RE.captures(line).unwrap();
        let filename = PathBuf::from(caps.get(1).unwrap().as_str());
        let expected_digest = parse_digest_sha256(caps.get(2).unwrap().as_str())?;
        Ok((filename, expected_digest))
    } else {
        Err(format!("fail to parse line: {}", line).into())
    }
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
