use super::device::Device;

pub trait Scanner {
    fn scan(&self) -> Vec<Device>;
}
