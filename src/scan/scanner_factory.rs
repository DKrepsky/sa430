/// A factory to create a scanner for the current OS.
///
/// # Returns
/// - a scanner for the current OS;
/// - an error if the OS is not supported.
pub struct ScannerFactory;

impl ScannerFactory {
    /// Creates a scanner for the current OS.
    #[allow(unreachable_code)]
    pub fn create() -> Result<Box<dyn Scanner>, String> {
        #[cfg(target_os = "linux")]
        return Ok(Box::new(LinuxScanner::new()));

        Err("No scanner for current OS".to_string())
    }
}
