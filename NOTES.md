# Notes

## Scanner integration tests

Although simulating a Serial Port is an easy task, to do so using a USB Serial Port is not so trivial.
So in order to create integration tests for the scanner some more advanced approach would be needed.
Some references used the USBIP tool for linux, while others used the Linux Gadget Driver, but it seems overkill for this project. For documentation purposes only, bellow are some references os how to implement it:
- https://github.com/smulikHakipod/USB-Emulation
- https://github.com/ckb-next/FaKeyboard
- https://github.com/lcgamboa/USBIP-Virtual-USB-Device
- https://github.com/xairy/raw-gadget?tab=readme-ov-file
- https://github.com/toasterllc/VirtualUSBDevice
- https://github.com/partizand/usb-vhci-libusb



## Coverage report 

Generate coverage report to use with [Coverage Gutters](https://marketplace.visualstudio.com/items?itemName=ryanluker.vscode-coverage-gutters).

```
cargo llvm-cov --lcov --output-path lcov.info
```