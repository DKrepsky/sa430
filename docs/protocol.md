# SA430 Serial Protocol

This document describes the SA430 Serial Protocol used for communication between a device and a computer via USB. It includes details on serial port settings, the structure of communication frames, and the process of transmitting and receiving frames. The document also lists various command codes, error codes, and provides a Rust implementation for calculating CRC16 checksums. The protocol ensures reliable data exchange by using a state machine to validate frames and handle errors.

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
| Sample   | 0x2A  | 0x02   | 0x0A | 0x05, 0x02 | 0xAD, 0xBF |

**Table 2:** Sample frame for a CMD_SET_GAIN command.

**Note:** all fields are in _big endian_ notation, including the CRC.

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

When a command returns data, it receives an ACK and then the data frames, as represented bellow.

```text
**Computer**                                    **Device**
    |--- [ Send CMD_GET_CORE_VER ] ------------> Blinks the led
    <------------------------------ [ ACK ] ---| Acknowledge
    <------------------------- [ Response ] ---| Respond with core version
```
**Figure 3:** Response with data.


If an error occurs, a NACK response is returned, together with the error code. The NACK has always a command with CMD_GET_LAST_ERROR and 2 data bytes with the erro code from Appendix B.

| Magic | Length | Cmd  | Data       | Crc        |
| ----- | ------ | ---- | ---------- | ---------- |
| 0x2A  | 0x02   | 0x06 | 0x03, 0x26 | 0x0f, 0x38 |

**Table 4:** Sample NACK frame for a CRC error.

```text
**Computer**                                    **Device**
    |--- [ Send CMD_BLINK_LED ] ----------------> Something is wrong
    <------------------------------ [ NACK ] ---| Return error
```
**Figure 4:** Response with error.

### Error Handling

To ensure reliable communication:
- Use CRC16 checks to detect frames with invalid checksums.
- If the length field differs from the received data size, discard the frame and send a “Length Error” code.
- Implement a timeout (e.g., 1 second). If the device does not respond within this period, clear buffers and indicate a “Timeout Error.”
- Use the returned error codes to detect when something goes wrong and its reason.

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
   3. Serial number not empty
   4. IDN not empty

If all check passes, the device is supported and in good state. The next step is to load the calibration data.

### Reading the calibration data

The calibration data is a series of parameters measured at the factory that will be applied to the measurements
in order to increase the device accuracy. It is unique per device and is stored in the flash memory at a predefined
address.

Reading the flash can be done with the CMD_FLASH_READ by passing the address and size, as shown in Figure 5.
Since each frame consists of a max of 255 bytes of data, to read more than 255 it is necessary to break the read
operation into multiple commands, were each will read `size / 255` blocks plus a final block with the remaining bytes.

```text
**Computer**                                                                            **Device**
    |--- [0x2A, 0x04, CMD_FLASH_READ, ADDR_H, ADDR_L, SIZE_H, SIZE_L, CRC_H, CRC_L] ---> Receive read command
    <---------------------------------------------------------------------- [ ACK ] ---| Acknowledge
    <----------------------------------------------------------------- [ Response ] ---| Return data from flash
```
**Figure 5:** Reading data from flash.

The flash is laid out as shown in Table 5, totalling 1671 bytes.

| Address | Field          | Type   | Notes          |
| ------- | -------------- | ------ | -------------- |
| 0xD400  | mem_start_addr | u16    | Must be 0xD400 |
| 0xD402  | mem_length     | u16    |                |
| 0xD404  | mem_type       | u16    | Must be 0x003E |
| 0xD406  | type_version   | u16    | Must be 0x0002 |
| 0xD408  | crc16          | u16    |                |
| 0xD40A  | CalData        | struct | See Table 6    |

**Table 5:** Flash memory layout.

The calibration data, stored at address 0xD40A has the format listed in Table 6. Note that all valies are in big endian
format, including the double ones.

| Field            | Type                | Notes                                                       |
| ---------------- | ------------------- | ----------------------------------------------------------- |
| format_version   | u16                 |                                                             |
| cal_date         | char[16]            |                                                             |
| sw_version       | u16                 |                                                             |
| prod_side        | u8                  |                                                             |
| frq_range        | FrequencyRange[3]   | FrequencyRange -> f_start: u32, f_stop: u32, f_samples: u32 |
| ref_lvl_table    | RefLevel[8]         | RefLevel -> value: i8, gain: u8                             |
| hardware_id      | u32                 |                                                             |
| serial_number    | char[16]            |                                                             |
| xtal_freq_hz     | u32                 |                                                             |
| xtal_freq_ppm    | u16                 |                                                             |
| cal_temp_start   | u8[6]               |                                                             |
| cal_temp_stop    | u8[6]               |                                                             |
| freq_gain_coeffs | FrequencyGain[3][8] | FrequencyGain -> dc_select: u8, values: double[8]           |

**Table 6:** Calibration data structure.

### RF settings

#### Frequency range

Before taking a measurement, there are several parameters that need to be configured. In this section we will take a
look on how this parameters are configured.

The first one is to select a frequency range, were SA430 hardware supports three ranges:
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

Recommended limits for BW are:
- 300-348 MHz: 0.1 MHz min to 48 MHz max
- 389-464 MHz: 0.1 MHz min to 75 MHz max
- 779-928 MHz: 0.1 MHz min to 74.5 MHz max

The Sa430 hardware will receive only the Fstart, Fstop parameters, so they must be converted from Fcentral and BW.
To pass the frequencies to the device, first they must be compensated for cristal oscillator deviations.
This is done with the calibration data as follows.

```rust
// Compensate frequency
//
// # Arguments
// - freq: desired frequency in MHz.
// - cal_data: calibration data loaded from flash.
//
// # Returns
// - Compensated frequency register value
pub fn compensate_freq(freq: MHz, cal_data: &CalData) -> u32 {
    let mut compensated: u32;
    if cal_data {
        compensated = ((freq * 65536.0) / cal_data.xtal_freq_mhz()) as u32;
    } else {
        compensated = ((f_start * 0xffff as f64) / 26.0) as u32;
    }
    compensated & 0x00ff_ffff
}
```

Then, we can send the values to the device with the CMD_SET_F_START and CMD_SET_F_STOP. Note that both Fstart and Fstop
are 3 bytes only.

#### Filter step width (FSW)

Next, we need to define the resolution of the measurement, which is made by the filter step width ($F_text{step}$)
parameter. The FSW in a value in Hz that determine the distance between two samples and a lower FSW value will give a
better resolution, but will also take more time for the measurement to complete.

Fstep can be computed from the BW and the desired number of samples and must be corrected according to the selected RBW,
as will be seen in the next section.

$$
N\text{samples}=\frac{BW}{F\text{step}} + 1 = \frac{F\text{stop}-F\text{start}}{F\text{step}} + 1
$$

Before sending the Fstep to the device, the value must also be compensated for oscillator deviations with the
`compensate_freq` function. Then, it can be sent using the CMD_SET_F_STEP command.

#### Resolution bandwidth (RBW)

Resolution bandwidth ($RBW$), is the bandwidth of a band-pass filter applied to each
measurement point. In order to not lose information, the RBW must be at least $2\times{FSW}$.

$$
RBW = 2\times{FSW}
$$

Since it is a digital filter, it's  value must be one of the Frequencies in Table 7.
The RegValue and RegValueIf are the actual parameters to sento to the device.

| RBW (double) | RegValue (u8) | RegValueIf (u8) |
| ------------ | ------------- | --------------- |
| 58.000       | 240           | 8               |
| 67.700       | 224           | 7               |
| 81.300       | 208           | 7               |
| 101.600      | 192           | 8               |
| 116.100      | 176           | 7               |
| 135.400      | 160           | 7               |
| 162.500      | 144           | 8               |
| 203.100      | 128           | 8               |
| 232.100      | 112           | 8               |
| 270.800      | 96            | 10              |
| 325.000      | 80            | 11              |
| 406.300      | 64            | 10              |
| 464.300      | 48            | 12              |
| 541.700      | 32            | 13              |
| 650.000      | 16            | 16              |
| 812.500      | 0             | 18              |

**Table 7:** RBW frequency and register values.

The RBW value can be automatically computed, as is used in the Easy RF settings of the TI software. For this, first
compute the minimum RBW for a given frequency step:

```rust
const MIN_RBW_STEP: double = 0.1
const MAX_RBW_STEP: double = 0.5

const RBW_TABLE = ... // Data from Table 7.

// Compute optimal RBW and FSW for a given FSW
//
// # Return
// Tuple with (FSW_adjusted, RBW, RBWIndex)
//
fn calc_easy_rf(mut fsw: double) -> (double, double, u32) {
    let mut rbw = fsw*MIN_RBW_STEP;
    let mut rbw_index = 0;

    // Find best RBW
    for index in 0..15 {
        if rbw <= RBW_TABLE[index].freq {
            rbw = RBW_TABLE[index].freq;
            rbw_index = index;
            break;
        }
    }

    // Adjust optimal FSW
    if fsw > rbw*MAX_RBW_STEP {
        fsw = rbw*MAX_RBW_STEP;
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

RevLvl can go from -70 dBm up to -35 dBm in steps of -5 bBm, as shown in Table 8.

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




### Collecting measurements

After the RF setup, you can start a measurement sending the command CMD_GET_SPEC_NO_INIT. Then, the device will start
the sampling and return multiple frames (CMD_GET_SPEC_NO_INIT) with the captured data. After the measurement is
complete, a last frame is sent by the device with command code CMD_GET_LAST_ERROR and status ERR_NO_ERROR. If any error
occurs during the sampling, the device will send CMD_GET_LAST_ERROR with the corresponding error code.

For better accuracy, the samples must be compensated by an offset calculated with the calibration data.

First, we need to compute the frequency for each measurement point.

$$
f(n) = F_{start} + n\times{F_{step}}, \quad n = 0, 1, \ldots, N_\mathrm{samples}
$$

Then, we can compute the $\beta$ factor for correction.

$$
\beta(f) = \sum_{i=0}^{i=7}{\alpha[i]\times{f^i}}
$$

$\alpha$ is obtained from `cal_data.freq_gain_coeffs[freq_range][ref_level_index]` table, where freq rang is 0, 1 or 2
for the ranges 300-348 MHz, 389-464 MHz and 779-928 MHz, respectively, and `reference_level_index` is the index of the
current reference level, as defined in Table 8.

With the coefficients, we can apply them to the measurements in order to get the final spectrum power in dBm:

$$
P(n) = \frac{Sample(n)}{2}-\beta(f(n)) \space [dBm]
$$

With this, $f(n)$ is our X axis in Hz and $P(n)$ is our Y axis in dBm.

**Note:** since computing the values of $\beta$ is expensive, it is recommended to cache the results and only compute
when the RF settings change.

## Appendix A: Command codes

| Command                  | Value | Request | Response | Description                              |
| ------------------------ | ----- | ------- | -------- | ---------------------------------------- |
| **General Commands**     |       |         |          |                                          |
| CMD_GET_IDN              | 0x01  |         | c_str    | IDN                                      |
| CMD_GET_HW_SER_NR        | 0x02  |         | u32      | Hardware Serial Number                   |
| CMD_HW_RESET             | 0x03  |         |          | Hardware Reset (PUC)                     |
| CMD_BLINK_LED            | 0x04  |         |          | Identify hardware by blinking LED        |
| CMD_GET_CORE_VER         | 0x05  |         | u16      | Core version                             |
| CMD_GET_LAST_ERROR       | 0x06  |         | u16      | Error code                               |
| CMD_SYNC                 | 0x07  |         | u16      | Unknown                                  |
| **Spectrum Measurement** |       |         |          |                                          |
| CMD_GET_SPEC_VER         | 0x14  |         | u16      | Spec version                             |
| CMD_SET_F_START          | 0x15  | u8[3]   |          | Set Start Frequency `fstart`             |
| CMD_SET_F_STOP           | 0x16  | u8[3]   |          | Set Stop Frequency `fstop`               |
| CMD_SET_F_STEP           | 0x17  | u8[2]   |          | Set Step Frequency `fstep`               |
| CMD_SET_FRQ              | 0x18  | ?       |          | Unknown                                  |
| CMD_SET_RBW              | 0x19  | u8      |          | Set Rx Filter Bandwidth                  |
| CMD_SET_DAC              | 0x1A  | ?       |          | Set DC value for the balun (unknown)     |
| CMD_SET_GAIN             | 0x1B  | u8      |          | Set gain of the Rx path                  |
| CMD_SET_IF               | 0x1C  | u8      |          | Set Intermediate Frequency               |
| CMD_INIT_PARAMETER       | 0x1E  |         |          | Setup system for spectrum measurement    |
| CMD_GET_SPEC_NO_INIT     | 0x1F  |         | u8[]     | Measure spectrum with defined parameters |
| **Production Commands**  |       |         |          |                                          |
| CMD_GET_PROD_VER         | 0x3C  |         | u16      | Get prod version                         |
| CMD_SET_PROD_FW_INIT     | 0x3D  | ?       |          | Unknown                                  |
| CMD_GET_TEMP             | 0x3E  |         | u8[6]    | Unknown                                  |
| CMD_SET_HW_ID            | 0x3F  | u32     |          | Set hardware id                          |
| CMD_GET_HW_ID            | 0x40  |         | u32      | Get Hardware id                          |
| CMD_GET_BOOT_CNT         | 0x41  |         | u32      | Boot count                               |
| CMD_SET_FOUT             | 0x42  | u8      |          | 0=Off, 1=26MHz, 2=RF Freq.  (next bytes) |
| CMD_SET_FXTAL            | 0x43  | u[12]   |          | Set frequency, incl. temp/cal versions   |
| CMD_GET_FXTAL            | 0x44  | u[12]   |          | Get frequency, incl. temp/cal versions   |
| CMD_SWEEP_EDC            | 0x45  |         | ?        | f, gain, repetition count                |
| CMD_GET_CHIP_TLV         | 0x49  |         | u8[8]    | Unknown                                  |
| **Flash Commands**       |       |         |          |                                          |
| CMD_FLASH_READ           | 0x0A  | u16,u16 | u8[]     | Send address and size, get flash content |
| CMD_FLASH_WRITE          | 0x0B  |         |          | Unknown                                  |
| CMD_FLASH_ERASE          | 0x0C  |         |          | Unknown                                  |
| CMD_FLASH_GET_CRC        | 0x0D  |         |          | Unknown                                  |
| **Deprecated**           |       |         |          |                                          |
| CMD_FRAME_ERROR          | 0xFF  | N/A     |          | Frame Error                              |

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
/// * `raw` - Raw frame
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
