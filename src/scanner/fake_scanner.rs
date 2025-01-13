use super::device::Device;
use super::scanner::Scanner;

pub struct FakeScanner {
    vec: Vec<Device>,
}

impl FakeScanner {
    pub fn new() -> Self {
        let vec = vec![
            Device::new("/dev/ttyUSB1", "08FF41E50F8B3A34", "0104"),
            Device::new("/dev/ttyUSB2", "08FF41E50F8B3A35", "0104"),
            Device::new("/dev/ttyUSB3", "08FF41E50F8B3A36", "0102"),
        ];
        FakeScanner { vec }
    }

    #[allow(dead_code)]
    pub fn new_empty() -> Self {
        FakeScanner { vec: vec![] }
    }
}

impl Scanner for FakeScanner {
    fn scan(&self) -> Vec<Device> {
        self.vec.clone()
    }
}
