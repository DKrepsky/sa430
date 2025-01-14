use super::scanner::Scanner;

#[cfg(target_os = "linux")]
use super::linux_scanner::LinuxScanner;

pub struct ScannerFactory;

/// A factory to create a scanner for the current OS.
impl ScannerFactory {
    /// Creates a scanner for the current OS.
    #[allow(unreachable_code)]
    pub fn create() -> Result<Box<dyn Scanner>, String> {
        #[cfg(target_os = "linux")]
        return Ok(Box::new(LinuxScanner::new()));

        return Err("No scanner for current OS".to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(target_os = "linux")]
    fn given_target_is_linux_when_create_then_create_a_linux_scanner() {
        let result = ScannerFactory::create();
        assert!(result.is_ok());
    }

    #[test]
    #[cfg(not(target_os = "linux"))]
    fn given_target_is_unknown_when_create_then_panic() {
        assert!(ScannerFactory::create().is_err());
    }
}
