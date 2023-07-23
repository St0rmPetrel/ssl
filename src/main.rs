use clap::{Parser, Subcommand};

mod md5;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    MD5(md5::MD5),
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::MD5(cmd) => cmd.hash(),
    }
}
