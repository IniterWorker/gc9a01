## ESP32 Playground

![ESP32](../images/esp32.jpg)

## Getting Started

### Toolchain

```bash
rustup target add xtensa-esp32-none-elf
```

<https://esp-rs.github.io/book/>

### Run under release

```sh
cargo run --example esp32_test --release --target xtensa-esp32-none-elf
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the
work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
