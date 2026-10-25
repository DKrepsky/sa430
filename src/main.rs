mod cli;

use std::process::ExitCode;

use clap::{Parser, Subcommand};

use cli::capture::capture;
use cli::{blink::blink, info::info, reboot::reboot, scan::scan, watch::watch, CliResult};
use sa430::{create_monitor, create_scanner, Sa430, SweepRequest};

#[derive(Parser)]
#[command(version)]
#[command(about = "Sa430 Command Line Interface Utility")]
#[command(subcommand_required = true, arg_required_else_help = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Scan for connected SA430 devices")]
    #[command(short_flag = 's')]
    Scan,

    #[command(about = "Monitor for connected SA430 devices")]
    #[command(short_flag = 'w')]
    Watch,

    #[command(about = "Read device information")]
    #[command(short_flag = 'i')]
    Info {
        #[arg(help = "Serial port to use")]
        port: String,
    },

    #[command(about = "Blink the LED on the device")]
    #[command(short_flag = 'b')]
    Blink {
        #[arg(help = "Serial port to use")]
        port: String,
    },

    #[command(about = "Performs a hardware reset on the device")]
    #[command(short_flag = 'r')]
    Reboot {
        #[arg(help = "Serial port to use")]
        port: String,
    },

    #[command(about = "Measure the spectrum and print it as CSV (frequency in MHz, power in dBm)")]
    #[command(short_flag = 'c')]
    Capture {
        #[arg(help = "Serial port to use")]
        port: String,
        #[arg(long, help = "The frequency to start capturing at, in MHz")]
        fstart: f64,
        #[arg(long, help = "The frequency to stop capturing at, in MHz")]
        fstop: f64,
        #[arg(long, help = "The frequency to step by, in MHz")]
        fstep: f64,
        #[arg(long = "rlevel", allow_negative_numbers = true)]
        #[arg(help = "Maximum signal power before saturation, in dBm. Default is -35 dBm")]
        #[arg(long_help = "Must be one of -35, -40, -45, -50, -55, -60, -65 or -70 dBm")]
        ref_level: Option<i8>,
    },
}

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {}", error);
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> CliResult {
    let mut stdout = std::io::stdout();

    match cli.command {
        Commands::Scan => scan(create_scanner().as_ref(), &mut stdout),
        Commands::Watch => watch(create_monitor()?.as_mut(), &mut stdout),
        Commands::Info { port } => info(&mut Sa430::open(&port)?, &mut stdout),
        Commands::Blink { port } => blink(&mut Sa430::open(&port)?, &mut stdout),
        Commands::Reboot { port } => reboot(&mut Sa430::open(&port)?, &mut stdout),
        Commands::Capture {
            port,
            fstart,
            fstop,
            fstep,
            ref_level,
        } => {
            let request = SweepRequest::new(fstart, fstop, fstep, ref_level)?;
            capture(&mut Sa430::open(&port)?, &request, &mut stdout)
        }
    }
}
