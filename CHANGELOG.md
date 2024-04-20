# Changelog

Rust implementation SPI 4-wire driver for the [`Gc9a01`](https://crates.io/crates/gc9a01-rs) display driver with a generic interface for display drivers and embedded-hal v1.0.0 support.

<!-- next-header -->

## [Unreleased] - ReleaseDate

## [0.2.0] - 2024-04-19

### Changed

- __(breaking)__ Support `embedded-hal: 1.0.0` .
- __(breaking)__ Support `display-interface: 0.5.0` support. Moving `CS` to `embedded_hal::spi::SpiDevice`.

### Removed

- Move playgrounds outside the library project.

<!-- next-url -->
[unreleased]: https://github.com/IniterWorker/gc9a01/compare/0.2.0...HEAD

[0.2.0]: https://github.com/IniterWorker/gc9a01/compare/0.1.0...0.2.0