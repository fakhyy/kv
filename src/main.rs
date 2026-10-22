mod cli;
mod commands;
mod database;
mod error;
mod password;
mod paths;
mod storage;

use clap::Parser;
use cli::Cli;

fn main() {
    let cli = Cli::parse();

    if let Err(err) = commands::run(cli) {
        eprintln!("kv: {err:#}");
        std::process::exit(1);
    }
}
