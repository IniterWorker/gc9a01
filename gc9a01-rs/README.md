# Gc9a01-rs driver

Rust implementation SPI 4-wire driver for the Gc9a01 display driver

<img src="../images/picture.jpg" alt="Gc9a01-rs" align="center">

## Getting started

- [ESP32 Example](../esp32-test/README.md)
- [STM32 Blackpill Example](../blackpill-test/README.md)

## Support

- [Embedded-graphics 2D graphics library](https://github.com/embedded-graphics/embedded-graphics)

## Drivers

- [gc9a01](./gc9a01)

## Hardware

- [1.28 inch TFT LCD Display Module Round RGB 240*240 GC9A01 Driver 4 Wire SPI](https://www.aliexpress.com/item/1005001382069930.html)

## TODO

- [x] Blackpill Playground
  - [x] SPI Setup
- [x] Buy the hardware
  - [x] Delivery expected by 13 Dec 2022
- [x] Implement write-only cmds
  - [x] Write-only operation
  - [x] Implement a working display init
  - [x] Documentation
  - [x] Test
- [ ] Implement display rotation
- [x] Implement buffered context
- [x] Implement embedded-graphics
- [ ] Rust Documentation
- [ ] Rust Cargo Deploy

## Inspiration

- From [jamwaffles/ssd1306](https://github.com/jamwaffles/ssd1306)

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