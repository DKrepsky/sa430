mod fixtures;
mod services;

use sa430::scanner_factory;
use services::scan::scan;
use std::env::consts::OS;

fn main() {
    scan(scanner_factory::create(OS), &mut std::io::stdout());
}
