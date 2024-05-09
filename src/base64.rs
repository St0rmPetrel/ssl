mod encoder;
mod new_liner;

use anyhow::Result;
use clap::Args;
use std::io;
use std::path;

use crate::libs::input;

#[derive(Args)]
pub struct Base64 {
    #[arg(short, long)]
    decode: bool,

    file: Option<path::PathBuf>,
}

impl Base64 {
    pub fn exec(self) -> Result<()> {
        let f = self.file.unwrap_or(path::PathBuf::from("-"));
        let mut input = input::Input::new(&f)?;

        let output = io::stdout().lock();

        if self.decode {
            println!("base64 decode");
        } else {
            let new_liner = new_liner::NewLiner::with_line_size(76, output);
            let mut encoder = encoder::Encoder::new(new_liner);

            if let Err(err) = io::copy(&mut input, &mut encoder) {
                eprintln!("{}", err);
            }
            if let Err(err) = encoder.finish() {
                eprintln!("{}", err);
            }
            println!("");
        }
        Ok(())
    }
}
