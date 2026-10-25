pub mod blink;
pub mod capture;
pub mod info;
pub mod reboot;
pub mod scan;
pub mod watch;

/// Result of a CLI command. Errors are printed by `main`.
pub type CliResult = Result<(), Box<dyn std::error::Error>>;

/// Placeholder printed for values a platform does not report.
pub const UNKNOWN: &str = "-";
