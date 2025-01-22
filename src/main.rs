mod cli;

use std::cell::RefCell;
use std::io::Write;
use std::rc::Rc;

use clap::{Parser, Subcommand};
use cli::scan::scan;
use cli::watch::watch;

use sa430::create_monitor;
use sa430::create_scanner;

#[derive(Parser)]
#[command(version)]
#[command(about = "Sa430 Command Line Interface Utility")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Scan for connected SA430 devices")]
    #[command(short_flag = 's')]
    Scan {},

    #[command(about = "Monitor for connected SA430 devices")]
    #[command(short_flag = 'w')]
    Watch {},
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Scan {}) => exec_scan(),
        Some(Commands::Watch {}) => exec_watch(),
        None => panic!("No command provided, use --help for usage"),
    }
}

fn exec_scan() {
    scan(create_scanner(), &mut std::io::stdout()).unwrap();
}

fn exec_watch() {
    let output: Rc<RefCell<dyn Write>> = Rc::new(RefCell::new(std::io::stdout()));
    watch(create_monitor().as_mut(), Rc::downgrade(&output)).unwrap()
}
