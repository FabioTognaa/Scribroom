use clap::Parser;
use scrib::cli::{self, Cli};

fn main() {
    let cli = Cli::parse();
    scrib::logging::init(cli.verbose);

    if let Err(error) = cli::execute(cli.command) {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}
