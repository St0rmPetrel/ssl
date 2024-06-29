use std::error;
use ssl::Cli;

type Result<T> = std::result::Result<T, Box<dyn error::Error>>;


fn main() -> Result<()> {
    let cli = Cli::new();

    cli.run()
}
