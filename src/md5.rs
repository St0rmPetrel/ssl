use clap::Args;

#[derive(Args)]
pub struct MD5 {}

impl MD5 {
    pub fn hash(&self) {
        println!("Hello, md5!")
    }
}
