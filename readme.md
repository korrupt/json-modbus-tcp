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
| SYS_TOT_POWER                     | 40003   | 4      | uint64 |
| SYS_TOT_KPOWER                    | 40007   | 4      | uint64 |
| SYS_TOT_CAPASITY                  | 40011   | 4      | uint64 |
| SYS_TOT_SOC                       | 40015   | 4      | uint64 |
| SYS_REQ_SETPOINT                  | 40019   | 4      | uint64 |
| SYS_AVAILABLE_POWER               | 40023   | 4      | uint64 |
| AMS_KW (**WIP**)                  | 40100   | 4      | uint64 |
| CMD_REQ_SETPOINT                  | 40200   | 4      | int64  |

## Compile production binary
`cargo build --release`

## Running the compiled binary
To view help, run `rust_modbus -help`

Regular running is done with `rust_modbus <PORT>`

## Running as dev
This is done with `cargo run`

## Running tests
Easily done with `cargo run test`
