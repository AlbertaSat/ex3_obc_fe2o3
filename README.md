# ex2_obc_fe2o3
Prototype OBC framework written in Rust

To build:

```bash
cargo build
```

To run on `localhost`, listening on port `<port>`:

```bash
target/debug/ex3_obc_fe203 --port <port>
```

You can also use `cargo run` if you prefer.

Once running, you can send commands to the OBC payloads and receive a response.
The command and response packets for the same format:

| Byte 0 | Byte 1 | Byte 2 | Byte 3 | Byte 4 | ... | Byte 63 |
| :----: | :----: | :----: | :----: | :----: | :----: | :----: |
| Length | Dest   | Opcode | Data 0 | Data 1 | ... | 0 |

where the _Length_ is the total number of bytes of data including the header,
_Dest_ is the payload (1 for `EPS`, 2 for `ADCS`, 3 for `DFGM`), _Opcode_ is
the payload specific command code, and _Data i_ is the payload and command
specific arguments.

For replies the _Dest_ and _Opcode_ are the same as the command, and the
optional response status and data start at _Data 0_.


