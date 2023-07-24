use clap::Args;
use std::{fmt, path::PathBuf};

#[derive(Args)]
pub struct MD5 {
    /// Files to digest (optional; default is stdin).
    /// With no FILE, or when FILE is -, read standard input.
    file: Option<Vec<PathBuf>>,
}

impl MD5 {
    pub fn exec(&self) {
        match &self.file {
            Some(files) => {
                for file in files.iter() {
                    process_file(file)
                }
            }
            None => process_file(&PathBuf::from("-")),
        };
    }
}

fn process_file(file: &PathBuf) {
    let data = read_file(file);
    let digest = compute(data);

    println!("{} {}", digest, file.display())
}

fn read_file(_file: &PathBuf) -> Vec<u8> {
    unimplemented!()
}

pub struct Digest(pub [u8; 16]);

impl fmt::Display for Digest {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        unimplemented!()
    }
}

pub fn compute<T: AsRef<[u8]>>(_data: T) -> Digest {
    unimplemented!()
}
