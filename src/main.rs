use anyhow::Result;

use ssl::Cli;

fn main() -> Result<()> {
    let cli = Cli::new();

    cli.run()
}
