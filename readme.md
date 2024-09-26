## Modbus TCP server for Bryte
Used as a proxy between the Bryte stack and TGN stack. Writte in async rust.

## Register Definition
| Name                              | Address | Length | Type   |
| --------------------------------- | ------- | ------ | ------ |
| **Coils** (Read/Write)            |
| Battery ON/OFF                    | 00001   | 1 bit  | bit    |
| MQTT Command Acknowledged         | 00100   | 1 bit  | bit    |
| **Holding Registers** (Read/Write)|
| SYS_STATE                         | 40001   | 1      | uint16 |
| SYS_LAST_STATE                    | 40002   | 1      | uint16 |
| SYS_TOT_POWER                     | 40003   | 1      | uint16 |
| SYS_TOT_KPOWER                    | 40004   | 1      | uint16 |
| SYS_TOT_CAPASITY                  | 40005   | 1      | uint16 |
| SYS_TOT_SOC                       | 40006   | 1      | uint16 |
| SYS_REQ_SETPOINT                  | 40007   | 1      | uint16 |
| SYS_AVAILABLE_POWER               | 40008   | 1      | uint16 |
| SYS_SOLAR_1_W                     | 40009   | 1      | uint16 |
| SYS_SOLAR_2_W                     | 40010   | 1      | uint16 |
| SYS_SOLAR_3_W                     | 40011   | 1      | uint16 |
| SYS_SOLAR_4_W                     | 40012   | 1      | uint16 |
| SYS_BUILDING_IN_W                 | 40013   | 1      | uint16 |
| AMS_KW (**WIP**)                  | 40100   | 1      | uint16 |
| CMD_REQ_SETPOINT_KW               | 40200   | 1      | int16  |

## Compile production binary
`cargo build --release`

## Running the compiled binary
To view help, run `rust_modbus -help`

Regular running is done with `rust_modbus <TARGET> [-p <PORT>]`

## Running as dev
This is done with `cargo run`

## Running tests
Easily done with `cargo run test`
