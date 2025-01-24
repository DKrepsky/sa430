mod cli;

use clap::{Parser, Subcommand};
use cli::capture::capture;
use cli::capture::CaptureParams;
use std::cell::RefCell;
use std::error::Error;
use std::io::Write;
use std::rc::Rc;

use cli::blink::blink;
use cli::info::info;
use cli::reboot::reboot;
use cli::scan::scan;
use cli::watch::watch;

use sa430::channel::SerialPortChannel;
use sa430::create_monitor;
use sa430::create_scanner;
use sa430::device::Sa430;

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

    #[command(about = "Read device information")]
    #[command(short_flag = 'i')]
    Info {
        #[arg(help = "The port to read device information from")]
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

    #[command(about = "Capture a spectrum")]
    #[command(short_flag = 'c')]
    Capture {
        #[arg(help = "Serial port to use")]
        port: String,
        #[arg(long)]
        #[arg(help = "The frequency to start capturing at, in MHz")]
        fstart: f64,
        #[arg(long)]
        #[arg(long, help = "The frequency to stop capturing at, in MHz")]
        fstop: f64,
        #[arg(long)]
        #[arg(long, help = "The frequency to step by, in MHz")]
        fstep: f64,
        #[arg(long = "rlevel")]
        #[arg(help = "Maximum signal power before saturation, in dBm. Default is -35 dBm")]
        #[arg(long_help = "Must be one of -35,-40, -45, -50, -55, -60, -65 or -70 dBm")]
        ref_level: Option<i8>,
    },
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Scan {}) => exec_scan(),
        Some(Commands::Watch {}) => exec_watch(),
        Some(Commands::Info { port }) => exec_info(&port),
        Some(Commands::Blink { port }) => exec_blink(&port),
        Some(Commands::Reboot { port }) => exec_reboot(&port),
        Some(Commands::Capture {
            port,
            fstart,
            fstop,
            fstep,
            ref_level,
        }) => exec_capture(&port, fstart, fstop, fstep, ref_level),
        None => panic!("No command provided, use --help for usage"),
    }
}

fn exec_scan() -> Result<(), Box<dyn Error>> {
    scan(create_scanner(), &mut std::io::stdout())?;
    Ok(())
}

fn exec_watch() -> Result<(), Box<dyn Error>> {
    let output: Rc<RefCell<dyn Write>> = Rc::new(RefCell::new(std::io::stdout()));
    watch(create_monitor().as_mut(), Rc::downgrade(&output))?;
    Ok(())
}

fn exec_info(port: &str) -> Result<(), Box<dyn Error>> {
    let channel = SerialPortChannel::new(port)?;
    let mut device = Sa430::new(Box::new(channel));
    info(&mut device, &mut std::io::stdout())
}

fn exec_blink(port: &str) -> Result<(), Box<dyn Error>> {
    let channel = SerialPortChannel::new(port)?;
    let mut device = Sa430::new(Box::new(channel));
    blink(&mut device, &mut std::io::stdout())
}

fn exec_reboot(port: &str) -> Result<(), Box<dyn Error>> {
    let channel = SerialPortChannel::new(port)?;
    let mut device = Sa430::new(Box::new(channel));
    reboot(&mut device, &mut std::io::stdout())
}

fn exec_capture(port: &str, fstart: f64, fstop: f64, fstep: f64, ref_level: Option<i8>) -> Result<(), Box<dyn Error>> {
    let channel = SerialPortChannel::new(port)?;
    let mut device = Sa430::new(Box::new(channel));
    let params = CaptureParams {
        fstart,
        fstop,
        fstep,
        ref_level,
    };
    capture(&mut device, &params, &mut std::io::stdout())
}
