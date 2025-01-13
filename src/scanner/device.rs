#[derive(Debug, Clone)]
pub struct Device {
    port: Box<str>,
    serial: Box<str>,
    version: Box<str>,
}

impl Device {
    pub fn new(port: &str, serial: &str, version: &str) -> Self {
        Device {
            port: port.into(),
            serial: serial.into(),
            version: version.into(),
        }
    }

    pub fn port(&self) -> &str {
        &self.port
    }

    pub fn serial(&self) -> &str {
        &self.serial
    }

    pub fn version(&self) -> &str {
        &self.version
    }
}
