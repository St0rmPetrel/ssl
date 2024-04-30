use anyhow::Result;
use clap::Args;

#[derive(Args)]
pub struct Base64 {
    #[arg(short, long)]
    decode: bool,
}

impl Base64 {
    pub fn exec(self) -> Result<()> {
        if self.decode {
            println!("base64 decode");
        } else {
            println!("base64 encode");
        }
        Ok(())
    }
}
