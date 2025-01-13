mod services;

use sa430::scanner::scanner_factory;
use services::scan::scan;

fn main() {
    scan(scanner_factory::create("linux"), &mut std::io::stdout());
}
