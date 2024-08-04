use ssl::Cli;

fn main() {
    let cli = Cli::new();

    if let Err(err) = cli.run() {
        eprintln!("{}", err);
        std::process::exit(1)
    }
}
