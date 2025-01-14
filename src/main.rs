mod commands;

use clap::{Parser, Subcommand};
use commands::scan::scan;

use sa430::scanner_factory;

#[derive(Parser)]
#[command(version)]
#[command(about = "Sa430 Command Line Interface Utility")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Scan for connected SA430 devices123")]
    #[command(short_flag = 's')]
    Scan {},
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Scan {}) => scan(scanner_factory::create().unwrap(), &mut std::io::stdout()),
        None => panic!("No command provided, use --help for usage"),
    }
}
