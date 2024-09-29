# Json Modbus
A fully customisable Modbus TCP server using only a JSON file and command line arguments, written in Rust for speed.

## Supported data types
The program supports several different data types, all of which can be defined using the <ins>[Python Format Characters](https://docs.python.org/3/library/struct.html#format-characters)</ins>. The following types are supported


| Format | C Type             | Rust type | Register Length (16bit) |
|:------:| ------------------ |:---------:|:-----------------------:|
|   h    | short              |    i16    |            1            |
|   H    | unsigned short     |    u16    |            1            |
|   i    | int                |    i32    |            2            |
|   I    | unsigned int       |    u32    |            2            |
|   q    | long long          |    i64    |            4            |
|   Q    | unsigned long long |    u64    |            4            |

## Register definition
The program will look for a `data.json` file in the same folder as itself. The registers can be defined as follows

<h5 a><strong><code>data.json</code></strong></h5>

```jsonc
{
    "1":       1,           // coils from 1-9999 (bit)
    "10005":   0,           // discrete inputs from 10001-19999 (bit)
    "30100":   0,           // input registers from 30001-39999
    "30101/h": -10,         
    "40001/I": 1000000,      // holding registers from 40001-49999
    "40003":   65535,       // offset one register as the previous spans two
    "40004/Q": 1000000000,
    "40008/q": -100000,
    "40012":   1,           // if no format is specified, 'H' is the default
    "40013":   2,
    "40014":   3,
    "40015":   4,
}
```

## Modbus quirks
Modbus in itself is not actually completely defined standard. As a result of this, in cases where one asks for "holding register 1", it is upto the implementation of said register to decide what "address 1" actually is. To better fit the various requirements, a "padding" command line argument will be added later, but as of now you have to ask for the full address, i.e. "40001" for "holding register #1".

## Credit
This software was made possible because of the efforts made in the [serde_json](https://github.com/serde-rs/json) and [tokio-modbus](https://github.com/slowtec/tokio-modbus) open source projects.

## Licence
[MIT](LICENSE)
