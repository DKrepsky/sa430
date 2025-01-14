use sa430::device::Device;
use sa430::scanner::Scanner;

#[allow(dead_code)]
pub(crate) struct FakeScanner {
    devices: Vec<Device>,
}

impl FakeScanner {
    #[allow(dead_code)]
    pub fn new(devices: Vec<Device>) -> Self {
        FakeScanner { devices }
    }
}

impl Scanner for FakeScanner {
    fn scan(&self) -> Vec<Device> {
        self.devices.clone()
    }
}
