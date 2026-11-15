# GC9A01

<p align="left">
    <a href="https://github.com/initerworker/gc9a01/actions/workflows/rust-release.yml"><img src="https://github.com/initerworker/gc9a01/actions/workflows/rust-release.yml/badge.svg" alt="Github CI/CD"></a>
    <a href="https://crates.io/crates/gc9a01-rs"><img src="https://img.shields.io/crates/v/gc9a01-rs.svg" alt="Crates.io"></a>
    <a href="https://docs.rs/gc9a01-rs"><img src="https://docs.rs/gc9a01-rs/badge.svg" alt="Docs.rs"></a>
</p>

<img src="./images/picture.jpg" alt="Gc9a01-rs" align="center">

Rust implementation SPI 4-wire driver for the Gc9a01 display driver with a generic interface for display drivers and embedded-hal v1.0.0 support.

## GC9A01 Display Driver

This crate provides a driver interface to the GC9A01 LCD display driver. It
supports SPI via the [`display_interface`](https://docs.rs/display_interface) crate.

The main driver is created using [`Gc9a01::new`] which accepts an interface instance, display,
size, rotation and mode. The following display modes are supported:

- [`mode::BasicMode`] - A simple mode with lower level methods available.
- [`mode::BufferedGraphics`] - A framebuffered mode with additional methods and integration with
  [embedded-graphics](https://docs.rs/embedded-graphics).

### Support

- [Embedded-graphics 2D graphics library](https://github.com/embedded-graphics/embedded-graphics)
- [Generic Interface for Display Drivers](https://github.com/therealprof/display-interface)
- [Embedded-hal v1.0.0](https://github.com/rust-embedded/embedded-hal/tree/embedded-hal-v1.0.0)

### Example

- [Waveshare esp32-s3-touch-lcd-1-28](https://github.com/IniterWorker/esp32-s3-touch-lcd-1-28)

### Buffered graphics example

```rust
use display_interface::{DataFormat, DisplayError, WriteOnlyDataCommand};
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
    text::Text,
};
use gc9a01::{display::DisplayResolution240x240, rotation::DisplayRotation, Gc9a01};

struct Stub;
impl WriteOnlyDataCommand for Stub {
    fn send_commands(&mut self, _cmd: DataFormat<'_>) -> Result<(), DisplayError> { Ok(()) }
    fn send_data(&mut self, _buf: DataFormat<'_>) -> Result<(), DisplayError> { Ok(()) }
}

let mut display = Gc9a01::new(Stub, DisplayResolution240x240, DisplayRotation::Rotate0)
    .into_buffered_graphics();
Rectangle::new(Point::new(10, 10), Size::new(50, 30))
    .into_styled(PrimitiveStyle::with_fill(Rgb565::RED))
    .draw(&mut display)
    .unwrap();
Text::new("Hi", Point::new(10, 60), MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE))
    .draw(&mut display)
    .unwrap();
display.flush().unwrap();
```

### Basic mode example

```rust
use display_interface::{DataFormat, DisplayError, WriteOnlyDataCommand};
use gc9a01::{display::DisplayResolution240x240, rotation::DisplayRotation, Gc9a01};

struct Stub;
impl WriteOnlyDataCommand for Stub {
    fn send_commands(&mut self, _cmd: DataFormat<'_>) -> Result<(), DisplayError> { Ok(()) }
    fn send_data(&mut self, _buf: DataFormat<'_>) -> Result<(), DisplayError> { Ok(()) }
}

let mut display = Gc9a01::new(Stub, DisplayResolution240x240, DisplayRotation::Rotate0);
display.set_pixel(10, 20, 0xF800).unwrap();
```

### Hardware

- [1.28 inch TFT LCD Display Module Round RGB 240*240 GC9A01 Driver 4 Wire SPI](https://www.aliexpress.com/item/1005001382069930.html)

## Development

A reproducible development environment is provided as a Nix flake
(stable Rust with clippy/rustfmt, the `thumbv7em-none-eabihf` stdlib for
the `no_std` cross check, `just`, and `cargo-readme`):

```sh
nix develop        # or: direnv allow (uses .envrc)
cargo test --features async
cargo clippy --all-targets --all-features -- -D warnings
cargo check --target thumbv7em-none-eabihf --all-features
```

`nix flake check` builds the crate and runs the test suites with
default and `async` features in the Nix sandbox. `nix build` /
`nix build .#async` produce the same checks as packages.

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
