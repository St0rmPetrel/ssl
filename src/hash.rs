mod md5;
mod sha256;

use anyhow::anyhow;
use anyhow::Context as Ctx;
use anyhow::Result;
use clap::Args;
use lazy_static::lazy_static;
use regex::Regex;
use std::io::BufRead;
use std::{fmt, io, path::PathBuf};

use crate::libs::hash;
use crate::libs::input;

#[derive(Args)]
pub struct Hash {
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

impl Hash {
    /// md5 command enter point.
    pub fn exec_md5(self) -> Result<()> {
        if self.check {
            self.check(HashAlgo::MD5)
        } else {
            self.create(HashAlgo::MD5)
        }
    }
    /// sha256 command enter point.
    pub fn exec_sha256(self) -> Result<()> {
        if self.check {
            self.check(HashAlgo::SHA256)
        } else {
            self.create(HashAlgo::SHA256)
        }
    }
}

impl Hash {
    /// read and check checksum file(s).
    /// compare for files listed in checksum file expected and actual computed hash of the file
    /// (among the list).
    fn check(self, algo: HashAlgo) -> Result<()> {
        let failed = self
            .file
            .unwrap_or(vec![PathBuf::from("-")])
            .into_iter()
            .map(|f| {
                let input = match input::Input::new(&f) {
                    Ok(input) => input,
                    Err(err) => {
                        eprintln!("{}", err);
                        return (f, 0, 1);
                    }
                };
                let failed = io::BufReader::new(input)
                    .lines()
                    .map(|l| check_line(&l?, &algo))
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
    fn create(self, algo: HashAlgo) -> Result<()> {
        // if no files in self.file add explicit stdin "-"
        let failed = self
            .file
            .unwrap_or(vec![PathBuf::from("-")])
            .into_iter()
            .map(|f| -> Result<()> {
                match algo {
                    HashAlgo::MD5 => {
                        let digest = md5_hash_file(&f)?;
                        print_file(&f, &algo, digest, self.tag);
                    }
                    HashAlgo::SHA256 => {
                        let digest = sha256_hash_file(&f)?;
                        print_file(&f, &algo, digest, self.tag);
                    }
                };
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

pub enum HashAlgo {
    MD5,
    SHA256,
}

impl fmt::Display for HashAlgo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            HashAlgo::MD5 => write!(f, "MD5"),
            HashAlgo::SHA256 => write!(f, "SHA256"),
        }
    }
}

// print file digest in specific format.
fn print_file<Digest: fmt::Display>(file: &PathBuf, algo: &HashAlgo, digest: Digest, is_bsd: bool) {
    let name = file.to_str().unwrap_or("-");
    if is_bsd {
        // BSD style checksum file
        println!("{} ({}) = {}", algo, name, digest)
    } else {
        // GNU style checksum file
        println!("{}  {}", digest, name)
    }
}

/// read file (could be stdin "-") calculate hash of the file data
fn md5_hash_file(file: &PathBuf) -> Result<md5::Digest> {
    let mut buf_r = input::Input::new(&file)
        .with_context(|| format!("fail to open {}", file.to_str().unwrap()))?;
    let ctx = md5::Context::new();

    let mut hasher = hash::Writer::new(ctx, hash::Endian::Little);
    io::copy(&mut buf_r, &mut hasher)
        .with_context(|| format!("fail to read {}", file.to_str().unwrap()))?;
    Ok(hasher.compute())
}

/// read file (could be stdin "-") calculate hash of the file data
fn sha256_hash_file(file: &PathBuf) -> Result<sha256::Digest> {
    let mut buf_r =
        input::Input::new(&file).with_context(|| format!("fail to open {}", file.to_str().unwrap()))?;
    let mut hasher = hash::Writer::new(sha256::Context::new(), hash::Endian::Big);
    io::copy(&mut buf_r, &mut hasher)
        .with_context(|| format!("fail to read {}", file.to_str().unwrap()))?;
    Ok(hasher.compute())
}

/// check line in checksum file
fn check_line(line: &str, algo: &HashAlgo) -> Result<()> {
    match algo {
        HashAlgo::MD5 => {
            let (file_name, expected_digest) = parse_md5_checksum_line(line)?;

            let actual_digest = match md5_hash_file(&file_name) {
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
        HashAlgo::SHA256 => {
            let (file_name, expected_digest) = parse_sha256_checksum_line(line)?;

            let actual_digest = match sha256_hash_file(&file_name) {
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
    }
}

fn parse_md5_checksum_line(line: &str) -> Result<(PathBuf, md5::Digest)> {
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
        let expected_digest = md5::Digest::from_str(caps.get(1).unwrap().as_str())?;
        Ok((filename, expected_digest))
    } else if BSD_STYLE_RE.is_match(line) {
        let caps = BSD_STYLE_RE.captures(line).unwrap();
        let filename = PathBuf::from(caps.get(1).unwrap().as_str());
        let expected_digest = md5::Digest::from_str(caps.get(2).unwrap().as_str())?;
        Ok((filename, expected_digest))
    } else {
        Err(anyhow::anyhow!("fail to parse line: {}", line))
    }
}

fn parse_sha256_checksum_line(line: &str) -> Result<(PathBuf, sha256::Digest)> {
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
        let expected_digest = sha256::Digest::from_str(caps.get(1).unwrap().as_str())?;
        Ok((filename, expected_digest))
    } else if BSD_STYLE_RE.is_match(line) {
        let caps = BSD_STYLE_RE.captures(line).unwrap();
        let filename = PathBuf::from(caps.get(1).unwrap().as_str());
        let expected_digest = sha256::Digest::from_str(caps.get(2).unwrap().as_str())?;
        Ok((filename, expected_digest))
    } else {
        Err(anyhow::anyhow!("fail to parse line: {}", line))
    }
}
