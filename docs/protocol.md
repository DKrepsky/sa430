# SA430 Serial Protocol

This document describes the SA430 Serial Protocol used for communication between a device and a computer via USB. It includes details on serial port settings, the structure of communication frames, and the process of transmitting and receiving frames. The document also lists various command codes, error codes, and provides a Rust implementation for calculating CRC16 checksums. The protocol ensures reliable data exchange by using a state machine to validate frames and handle errors.

The reference for this document is TI's original host software (the `SA430.Dll` driver and the `SA430.Gui` application,
mirrored at <https://github.com/pedrominatel/MSP-SA430-SUB1GHZ_Linux-GUI>). Where something is not backed by that
source, it is marked as such.

## Serial Port Settings

The device connects through USB as a serial port, identified by the vendor ID (VID="2047") and product ID (PID="0005").

| Setting      | Value    |
| ------------ | -------- |
| Baud         | 926100   |
| Data Bits    | 8        |
| Stop Bits    | 1        |
| Parity       | None     |
| Flow Control | Hardware |

**Table 1:** Serial port settings.

 Although hardware control is enabled, there is no evidence of its usage.

TI's driver opens the port with a 1000 ms read timeout. On top of that, it waits up to 100 ms for each ACK and response
frame, and up to 500 ms when reading flash.

## Communication protocol

### Frame structure

Communication between the device and the computer is conducted through packets called frames. Each frame consists of the following components:

- **Magic**: A constant value (0x2A) that denotes the start of a frame.
- **Length**: The size of the data field in bytes.
- **Command**: Indicates the command being sent or responded to.
- **Data**: The payload of the command or response.
- **CRC**: A CRC16 checksum of the frame to ensure data integrity.

| Field    | Magic | Length | Cmd  | Data       | Crc        |
| -------- | ----- | ------ | ---- | ---------- | ---------- |
| Index    | 0     | 1      | 2    | 3          | 3+N        |
| Size [B] | 1     | 1      | 1    | N          | 2          |
| Sample   | 0x2A  | 0x02   | 0x1B | 0x05, 0x02 | 0xC0, 0x4A |

**Table 2:** Sample frame for a CMD_SET_GAIN command.

**Note:** all fields are in _big endian_ notation, including the CRC. The only exception is the flash program header
(see Table 5), which is stored little-endian.

Each frame contains from 0 up to 255 bytes of data.
The data format depends on the command being sent/received.

The CRC must be calculated in accordance to Appendix C.

### Transmitting and receiving frames

The state machine continually reads bytes from the input FIFO until it completes a valid frame.
It looks for a Magic byte (0x2A), then reads the length, command, data, and CRC. If the CRC is correct, it stores the frame and signals a received event; otherwise, it signals an error.

```text
**Device**                                       **Computer**
    |-------------------------------------------> WAIT_MAGIC
    |--- Magic (0x2A) ---------------------------> WAIT_LENGTH
    |--- Length --------------------------------> WAIT_CMD
    |--- Command -------------------------------> if (length > 0) WAIT_DATA else WAIT_CRC_HIGH
    |--- Data bytes (Length - 1 times) ---------> while (received < length) WAIT_DATA
    |--- Data (Last byte) ----------------------> WAIT_CRC_HIGH
    |--- CRC High byte -------------------------> WAIT_CRC_LOW
    |--- CRC Low byte --------------------------> CRC OK ? Frame Event : Error Event
    |-------------------------------------------> WAIT_MAGIC (repeat)
```
**Figure 1:** Sequence diagram representing the communication between device and the computer.

### Executing commands

Command execution works in a request/response format, where the device can either acknowledge (ACK) or not acknowledge (NACK) a command.

If the command is executed successfully, an ACK is returned.

```text
Computer                                         Device
    |--- [ Send CMD_BLINK_LED ] ---------------> Blinks the led
    <------------------------------ [ ACK ] ---| Acknowledge
```
**Figure 2:** Request/response command.

The ACK packet consists of a frame with the same command, but with length 0, i.e, no data.

| Magic | Length | Cmd  | Crc        |
| ----- | ------ | ---- | ---------- |
| 0x2A  | 0x00   | 0x04 | 0xC5, 0xAC |

**Table 3:** ACK frame for a CMD_BLINK_LED command.

When a command returns data, it receives an ACK and then the data frames, as represented below.

```text
**Computer**                                    **Device**
    |--- [ Send CMD_GET_CORE_VER ] ------------> Read core version
    <------------------------------ [ ACK ] ---| Acknowledge
    <------------------------- [ Response ] ---| Respond with core version
```
**Figure 3:** Response with data.


If an error occurs, a NACK response is returned, together with the error code. The NACK always has the command
CMD_GET_LAST_ERROR and 2 data bytes with the error code from Appendix B.

| Magic | Length | Cmd  | Data       | Crc        |
| ----- | ------ | ---- | ---------- | ---------- |
| 0x2A  | 0x02   | 0x06 | 0x03, 0x29 | 0xFE, 0xD7 |

**Table 4:** Sample NACK frame for a CRC error (ERR_WRONG_CRC_LOW_BYTE).

```text
**Computer**                                    **Device**
    |--- [ Send CMD_BLINK_LED ] ----------------> Something is wrong
    <------------------------------ [ NACK ] ---| Return error
```
**Figure 4:** Response with error.

### Error Handling

TI's host software handles errors as follows:
- Frames with an invalid CRC are discarded and only signalled as a CRC error event. There is no retry.
- The length byte is trusted: the decoder reads exactly that many data bytes. A frame cut short by a lost byte is only
  detected by its CRC.
- Frames that are neither the expected ACK nor the expected response are skipped while waiting. If nothing matching
  arrives within the timeout (100 ms, or 500 ms for flash reads), the command fails.
- The error code returned in a NACK tells what went wrong and why.

## Device operation

### Initialization

After connecting to the device, a series of steps are done in order to verify the firmware integrity and version support.

1. Get core version (CMD_GET_CORE_VER)
2. Get hardware serial number (CMD_GET_HW_SER_NR)
3. Get IDN (CMD_GET_IDN)
4. Initialize the spectrum analyzer parameters (CMD_INIT_PARAMETER)
5. Get spectrum analyzer version (CMD_GET_SPEC_VER)
6. Check versions
   1. CoreVersion >= 0x0209 && CoreVersion != 0xffff
   2. SpecVersion >= 0x0204 && SpecVersion != 0xffff
7. Load the calibration data (see next section)
8. Blink the LED (CMD_BLINK_LED) to signal that the device is ready

If CMD_INIT_PARAMETER fails, TI's GUI reads the IDN again and asks for a firmware update. Separately, it runs the
firmware updater (`SA430_FW0205_SPEC.bin`, through an external `sa430FwUpdate.exe`) when
`CoreVersion >= 0x0209 && SpecVersion <= 0x0204`. Note that SpecVersion 0x0204 passes the check above but still gets
updated.

TI does not check that the serial number and IDN are non-empty, but it is a cheap extra sanity check.

### Reading the calibration data

The calibration data is a series of parameters measured at the factory that will be applied to the measurements
in order to increase the device accuracy. It is unique per device and is stored in the flash memory at a predefined
address.

Reading the flash can be done with the CMD_FLASH_READ by passing the address and size, as shown in Figure 5.
Since each frame consists of a max of 255 bytes of data, to read more than 255 it is necessary to break the read
operation into multiple commands, where each will read `size / 255` blocks plus a final block with the remaining bytes.

```text
**Computer**                                                                            **Device**
    |--- [0x2A, 0x04, CMD_FLASH_READ, ADDR_H, ADDR_L, SIZE_H, SIZE_L, CRC_H, CRC_L] ---> Receive read command
    <---------------------------------------------------------------------- [ ACK ] ---| Acknowledge
    <----------------------------------------------------------------- [ Response ] ---| Return data from flash
```
**Figure 5:** Reading data from flash.

**Note:** in TI's `cmdFlashRead`, SIZE is the number of bytes _still to read_ (not the chunk size), and the device
answers with at most 255 bytes per request. Sending the chunk size (≤ 255) also works. Read exactly the chunk size from
each response and ignore any extra bytes.

The calibration region starts at 0xD400 (`CALDATA_FLASH_START`) and ends at 0xEBFF (`CALDATA_FLASH_END`). It begins
with a 10-byte program header, followed by the 1671-byte calibration data, 1681 bytes in total. Table 5 shows the
layout. The header fields are **little-endian** (MSP430 native order).

| Address | Field          | Type   | Notes                                       |
| ------- | -------------- | ------ | ------------------------------------------- |
| 0xD400  | mem_start_addr | u16 LE | 0xD400 in the test fixture                  |
| 0xD402  | mem_length     | u16 LE | 0x06D4 in the test fixture                  |
| 0xD404  | mem_type       | u16 LE | Must be 0x003E (`PROGTYPE_CALC`, 62)        |
| 0xD406  | type_version   | u16 LE | 0x0200 in the test fixture                  |
| 0xD408  | crc16          | u16 LE |                                             |
| 0xD40A  | CalData        | struct | See Table 6                                 |

**Table 5:** Flash memory layout.

TI only checks `mem_type`. The "test fixture" values come from `test_utils::PROG_HEADER_DATA` and are not checked by
TI's software.

The calibration data, stored at address 0xD40A, has the format listed in Table 6. Unlike the header, all values are in
big endian format, including the double ones. The fields are packed (no padding), for a total of 1671 bytes.

| Field            | Type                | Notes                                                       |
| ---------------- | ------------------- | ----------------------------------------------------------- |
| format_version   | u16                 |                                                             |
| cal_date         | char[16]            |                                                             |
| sw_version       | u16                 |                                                             |
| prod_side        | u8                  |                                                             |
| frq_range        | FrequencyRange[3]   | FrequencyRange -> f_start: u32, f_stop: u32, f_samples: u32 |
| ref_lvl_table    | RefLevel[8]         | RefLevel -> value: i8, gain: u8                             |
| hardware_id      | u32                 |                                                             |
| serial_number    | char[16]            | TUSB3410 USB serial number                                  |
| xtal_freq_hz     | u32                 | Crystal frequency in Hz                                     |
| xtal_freq_ppm    | i16                 | Signed (`short` in TI's code)                               |
| cal_temp_start   | u8[6]               |                                                             |
| cal_temp_stop    | u8[6]               |                                                             |
| freq_gain_coeffs | FrequencyGain[3][8] | FrequencyGain -> dc_select: u8, values: double[8]           |

**Table 6:** Calibration data structure.

### RF settings

#### Frequency range

Before taking a measurement, there are several parameters that need to be configured. In this section we will take a
look at how these parameters are configured.

The first one is to select a frequency range, where SA430 hardware supports three ranges:
- 300 MHz to 348 MHz
- 389 MHz to 464 MHz
- 779 MHz to 928 MHz

Then, we can select either a central frequency ($F\text{central}$) and a bandwidth ($BW$) in MHz, or define a start
($F\text{start}$) and stop ($F\text{stop}$) frequency. For example, we can select the 389-464 MHz range, with a center
frequency of 433 MHz and a bandwidth of 20 MHz or a start frequency with 423 MHz to a stop frequency of 443 MHz.
To convert from one format to another, we have:

$$
F\text{start} = F\text{central} - \frac{BW}{2}
$$
$$
F\text{stop} = F\text{central} + \frac{BW}{2}
$$

TI's GUI clips $F\text{start}$ and $F\text{stop}$ to the limits of the selected range.

Recommended limits for BW are:
- 300-348 MHz: 0.1 MHz min to 48 MHz max
- 389-464 MHz: 0.1 MHz min to 75 MHz max
- 779-928 MHz: 0.1 MHz min to 74.5 MHz max

The Sa430 hardware will receive only the Fstart, Fstop parameters, so they must be converted from Fcentral and BW.
To pass the frequencies to the device, first they must be compensated for crystal oscillator deviations.
This is done with the calibration data as follows.

```rust
// Compensate frequency
//
// # Arguments
// - freq_mhz: desired frequency in MHz.
// - cal_data: calibration data loaded from flash, if available.
//
// # Returns
// - Frequency register value (not yet masked to the register width)
pub fn compensate_freq(freq_mhz: f64, cal_data: Option<&CalData>) -> u32 {
    match cal_data {
        Some(cal) => ((freq_mhz * 65536.0) / (cal.xtal_freq_hz as f64 / 1e6)) as u32,
        // TI's fallback without calibration data: 0xFFFF (not 65536) and a nominal 26 MHz crystal.
        None => ((freq_mhz * 0xffff as f64) / 26.0) as u32,
    }
}
```

Then, we can send the values to the device with the CMD_SET_F_START and CMD_SET_F_STOP. Note that both Fstart and Fstop
are 3 bytes only, so the register value is masked with `0x00ff_ffff`.

Because of this conversion, the frequencies the device actually uses are quantized. TI computes them back for display
and calibration ($F_\text{xtal}$ in MHz):

$$
F\text{actual} = \frac{F_\text{xtal}\times\left\lfloor\frac{F\times 65536}{F_\text{xtal}}\right\rfloor}{65536}
$$

#### Filter step width (FSW)

Next, we need to define the resolution of the measurement, which is made by the filter step width ($F\text{step}$)
parameter. The FSW is a value in **kHz** that determines the distance between two samples. A lower FSW value gives a
better resolution, but will also take more time for the measurement to complete.

Fstep can be computed from the BW and the desired number of samples and must be corrected according to the selected RBW,
as will be seen in the next section.

$$
N\text{samples}=\left\lfloor\frac{BW}{F\text{step}}\right\rfloor + 1 = \left\lfloor\frac{F\text{stop}-F\text{start}}{F\text{step}}\right\rfloor + 1
$$

Before sending the Fstep to the device, the value must be converted to MHz (divided by 1000) and also compensated for
oscillator deviations with the `compensate_freq` function. The result is truncated to a `u16` (not masked to 24 bits)
and sent as 2 bytes using the CMD_SET_F_STEP command.

#### Resolution bandwidth (RBW)

Resolution bandwidth ($RBW$), is the bandwidth of a band-pass filter applied to each
measurement point. In order to not lose information, the RBW must be at least $2\times{FSW}$.

$$
RBW = 2\times{FSW}
$$

Since it is a digital filter, its value must be one of the frequencies in Table 7.
The RegValue and RegValueIf are the actual parameters to send to the device.

| RBW [kHz] (double) | RegValue (u8) | RegValueIf (u8) |
| ------------------ | ------------- | --------------- |
| 58.000             | 240           | 8               |
| 67.700             | 224           | 7               |
| 81.300             | 208           | 7               |
| 101.600            | 192           | 8               |
| 116.100            | 176           | 7               |
| 135.400            | 160           | 7               |
| 162.500            | 144           | 8               |
| 203.100            | 128           | 8               |
| 232.100            | 112           | 8               |
| 270.800            | 96            | 10              |
| 325.000            | 80            | 11              |
| 406.300            | 64            | 10              |
| 464.300            | 48            | 12              |
| 541.700            | 32            | 13              |
| 650.000            | 16            | 16              |
| 812.500            | 0             | 18              |

**Table 7:** RBW frequency and register values.

The RBW value can be automatically computed, as is used in the Easy RF settings of the TI software. It starts from a
step that gives 180 points over the span, picks the smallest RBW that is at least 10% of that step, then limits the
step to half of the chosen RBW:

```rust
const INIT_FRQ_POINTS: f64 = 180.0;
const MIN_RBW_STEP: f64 = 0.1;
const MAX_RBW_STEP: f64 = 0.5;

const RBW_TABLE: [f64; 16] = [/* RBW column of Table 7, in kHz */];

// Compute the Easy RF step width and RBW for a span.
//
// # Arguments
// - f_start, f_stop: frequencies in MHz.
//
// # Return
// Tuple with (fsw_khz, rbw_khz, rbw_index)
fn calc_easy_rf(f_start: f64, f_stop: f64) -> (f64, f64, usize) {
    let mut fsw = (f_stop - f_start) / INIT_FRQ_POINTS * 1000.0;
    let rbw = fsw * MIN_RBW_STEP;

    // Find the smallest RBW that fits, falling back to the largest one (index 15).
    let rbw_index = RBW_TABLE[..15].iter().position(|&r| rbw <= r).unwrap_or(15);
    let rbw = RBW_TABLE[rbw_index];

    // Adjust FSW so that RBW >= 2 * FSW
    if fsw > rbw * MAX_RBW_STEP {
        fsw = rbw * MAX_RBW_STEP;
    }

    (fsw, rbw, rbw_index)
}
```

To update the RBW, first send a CMD_SET_RBW command, passing the RegValue as data, then send a CMD_SET_IF command
with RegValueIf as data.

#### Reference Level (RefLvl)

The last parameter is the reference level ($RefLvl$), that is defined as:
> The reference level sets the maximum power level that can be measured without saturating the measurement device. If a
power level higher than the reference level is applied, the measurement result contains signal artifacts
(power) close to the input signal frequency.

RefLvl can go from -70 dBm up to -35 dBm in steps of 5 dBm, as shown in Table 8.

| RefLevel (i8) | RegValue (u8) |
| ------------- | ------------- |
| -35           | 128           |
| -40           | 144           |
| -45           | 145           |
| -50           | 74            |
| -55           | 12            |
| -60           | 179           |
| -65           | 44            |
| -70           | 61            |

**Table 8:** Reference level values and corresponding register setting.

To update the reference level at the device use the command CMD_SET_GAIN, passing the corresponding RegValue as
parameter.

#### Command order

TI's GUI sends the RF settings in this order, waiting for the ACK of each command before sending the next one:

1. CMD_SET_F_START (3 bytes)
2. CMD_SET_F_STOP (3 bytes)
3. CMD_SET_F_STEP (2 bytes)
4. CMD_SET_RBW (1 byte)
5. CMD_SET_IF (1 byte)
6. CMD_SET_GAIN (1 byte)

If any command fails, the remaining ones are not sent.

### Collecting measurements

After the RF setup, you can start a measurement sending the command CMD_GET_SPEC_NO_INIT. The device ACKs it, then
starts the sampling and returns multiple data frames (CMD_GET_SPEC_NO_INIT) with the captured data. After the
measurement is complete, a last frame is sent by the device with command code CMD_GET_LAST_ERROR and status
ERR_NO_ERROR. If any error occurs during the sampling, the device will send CMD_GET_LAST_ERROR with the corresponding
error code.

**Note:** TI's GUI treats _any_ 2-byte CMD_GET_LAST_ERROR frame as the end of the sweep. Its check for ERR_NO_ERROR is
commented out.

Each data byte is one sample, a **signed** 8-bit value (`i8`). The samples of all data frames are concatenated in
order.

For better accuracy, the samples must be compensated by an offset calculated with the calibration data.

First, we need to compute the frequency for each measurement point, using the quantized $F\text{start}$ and
$F\text{step}$ the device actually uses (see $F\text{actual}$ above):

$$
f(n) = F_{start} + n\times{F_{step}}, \quad n = 0, 1, \ldots, N_\mathrm{samples} - 1
$$

Then, we can compute the $\beta$ factor for correction, with $f$ in **Hz**:

$$
\beta(f) = \sum_{i=0}^{i=7}{\alpha[i]\times{f^i}}
$$

$\alpha$ is obtained from `cal_data.freq_gain_coeffs[freq_range][ref_level_index].values`, where freq_range is 0, 1 or
2 for the ranges 300-348 MHz, 389-464 MHz and 779-928 MHz, respectively, and `ref_level_index` is the index of the
current reference level, as defined in Table 8.

With the coefficients, we can apply them to the measurements in order to get the final spectrum power in dBm:

$$
P(n) = \frac{Sample(n)}{2}-\beta(f(n)) \space [dBm]
$$

With this, $f(n)$ is our X axis in Hz and $P(n)$ is our Y axis in dBm. TI's GUI only applies the offset when the number
of received samples matches $N\text{samples}$.

**Note:** since computing the values of $\beta$ is expensive, it is recommended to cache the results and only compute
when the RF settings change.

## Appendix A: Command codes

| Command                  | Value | Request    | Response | Description                              |
| ------------------------ | ----- | ---------- | -------- | ---------------------------------------- |
| **General Commands**     |       |            |          |                                          |
| CMD_GET_IDN              | 0x01  |            | c_str    | IDN                                      |
| CMD_GET_HW_SER_NR        | 0x02  |            | u32      | Hardware Serial Number                   |
| CMD_HW_RESET             | 0x03  | 0xDE, 0xAD |          | Hardware Reset (PUC), requires password  |
| CMD_BLINK_LED            | 0x04  |            |          | Identify hardware by blinking LED        |
| CMD_GET_CORE_VER         | 0x05  |            | u16      | Core version                             |
| CMD_GET_LAST_ERROR       | 0x06  |            | u16      | Error code                               |
| CMD_SYNC                 | 0x07  |            |          | Sync, ACK only                           |
| **Spectrum Measurement** |       |            |          |                                          |
| CMD_GET_SPEC_VER         | 0x14  |            | u16      | Spec version                             |
| CMD_SET_F_START          | 0x15  | u8[3]      |          | Set Start Frequency `fstart`             |
| CMD_SET_F_STOP           | 0x16  | u8[3]      |          | Set Stop Frequency `fstop`               |
| CMD_SET_F_STEP           | 0x17  | u8[2]      |          | Set Step Frequency `fstep`               |
| CMD_SET_FRQ              | 0x18  | ?          |          | Unknown                                  |
| CMD_SET_RBW              | 0x19  | u8         |          | Set Rx Filter Bandwidth                  |
| CMD_SET_DAC              | 0x1A  | ?          |          | Set DC value for the balun (unknown)     |
| CMD_SET_GAIN             | 0x1B  | u8         |          | Set gain of the Rx path                  |
| CMD_SET_IF               | 0x1C  | u8         |          | Set Intermediate Frequency               |
| CMD_INIT_PARAMETER       | 0x1E  |            |          | Setup system for spectrum measurement    |
| CMD_GET_SPEC_NO_INIT     | 0x1F  |            | i8[]     | Measure spectrum with defined parameters |
| **Production Commands**  |       |            |          |                                          |
| CMD_GET_PROD_VER         | 0x3C  |            | u16      | Get prod version                         |
| CMD_SET_PROD_FW_INIT     | 0x3D  | ?          |          | Unknown                                  |
| CMD_GET_TEMP             | 0x3E  |            | u8[]     | Temperature, length unknown              |
| CMD_SET_HW_ID            | 0x3F  | u32        |          | Set hardware id                          |
| CMD_GET_HW_ID            | 0x40  |            | u32      | Get Hardware id                          |
| CMD_GET_BOOT_CNT         | 0x41  |            | u32      | Boot count                               |
| CMD_SET_FOUT             | 0x42  | u8         |          | 0=Off, 1=26MHz, 2=RF Freq.  (next bytes) |
| CMD_SET_FXTAL            | 0x43  | ?          |          | Set crystal frequency (4 bytes), plus temperature, cal. version and HW id |
| CMD_GET_FXTAL            | 0x44  |            | u32      | Get crystal frequency                    |
| CMD_SWEEP_EDC            | 0x45  | ?          | ?        | f, gain, repetition count                |
| CMD_GET_CHIP_TLV         | 0x49  |            | u8[]     | Chip TLV data, length unknown            |
| **Flash Commands**       |       |            |          |                                          |
| CMD_FLASH_READ           | 0x0A  | u16,u16    | u8[]     | Send address and size, get flash content |
| CMD_FLASH_WRITE          | 0x0B  |            |          | Unknown                                  |
| CMD_FLASH_ERASE          | 0x0C  |            |          | Unknown                                  |
| CMD_FLASH_GET_CRC        | 0x0D  |            |          | Unknown                                  |
| **Deprecated**           |       |            |          |                                          |
| CMD_FRAME_ERROR          | 0xFF  | N/A        |          | Frame Error                              |

The flash commands are only defined in TI's GUI (`drvSA430.h`, together with `FLASH_SEGMENT_SIZE = 512`), and only
CMD_FLASH_READ is used. Firmware updates are done by an external tool whose source is not available. The request and
response formats of CMD_GET_TEMP, CMD_GET_CHIP_TLV and CMD_GET_FXTAL come from how TI's GUI reads them. The other
production commands are not used by the GUI.

## Appendix B: Error codes

| Error Code                                   | Value  |
| -------------------------------------------- | ------ |
| ERR_NO_ERROR                                 | 0x0000 |
| ERR_CMD_BUFFER_OVERFLOW                      | 0x0320 |
| ERR_WRONG_CMD_LENGTH                         | 0x0321 |
| ERR_CMD_ABORTED                              | 0x0322 |
| ERR_LOST_CMD                                 | 0x0323 |
| ERR_CMD_UNKNOWN                              | 0x0324 |
| ERR_TOO_MUCH_DATA_REQUESTED_BY_USER_FUNCTION | 0x0325 |
| ERR_RESTORE_PROGRAM_COUNTER                  | 0x0326 |
| ERR_BUFFER_POS_OUT_OF_RANGE                  | 0x0327 |
| ERR_EEQ_BUFFER_OVERFLOW                      | 0x0328 |
| ERR_WRONG_CRC_LOW_BYTE                       | 0x0329 |
| ERR_WRONG_CRC_HIGH_BYTE                      | 0x032A |
| ERR_RESTORE_FROM_PACKET_ERROR                | 0x032C |
| ERR_NO_FRAME_START                           | 0x032D |
| ERR_WRONG_PKT_LENGTH                         | 0x032E |
| ERR_PACKET_INCOMPLETE                        | 0x032F |
| ERR_PACKET_ERROR                             | 0x0330 |
| ERR_STUPID_PACKET_HANDLER                    | 0x0331 |
| ERR_BUFFER_OVERFLOW                          | 0x0352 |
| ERR_BUFFER_UNDERRUN                          | 0x0353 |
| ERR_FLASH_NOT_ERASED                         | 0x044C |
| ERR_FLASH_MISMATCH                           | 0x044D |
| ERR_RSSI_VALID_FLAG_NOT_SET                  | 0x04B0 |
| ERR_PLL_NOT_SETTLED                          | 0x04B1 |

## Appendix C: CRC16 rust implementation

```rust
/// Calculate CRC16 of frame
///
/// # Arguments
///
/// * `raw` - Raw frame, including the 2 CRC bytes (their value is ignored)
///
/// # Returns
///
/// * `u16` - Generated CRC16 value
fn crc16(raw: &[u8]) -> u16 {
    let mut crc: u16 = 0x2A;
    let length = raw.len() - 2;

    for index in 1..length {
        crc = (crc >> 8) | (crc << 8);
        crc ^= raw[index] as u16;
        crc ^= (crc & 0xff) >> 4;
        crc ^= (crc << 8) << 4;
        crc ^= ((crc & 0xff) << 4) << 1;
    }
    crc
}
```
