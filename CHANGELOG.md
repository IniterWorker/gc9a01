# Changelog

Rust implementation SPI 4-wire driver for the [`Gc9a01`](https://crates.io/crates/gc9a01-rs) display driver with a generic interface for display drivers and embedded-hal v1.0.0 support.

<!-- next-header -->

## [0.5.0] - Unreleased

### Added

* Async driver `Gc9a01Async` behind feature `async`, with public async transport trait `AsyncWriteOnlyDataCommand` and `AsyncSpiInterface`
* `Command::encode` returning `EncodedCommand`, shared by blocking and async send paths
* Panel init sequence as data: `InitStep` and `DisplayDefinition::INIT_STEPS`
* Buffered mode accepts caller-provided framebuffer via `into_buffered_graphics_with_buffer`
* `Gc9a01::into_basic_mode` and `Gc9a01::release` (and async counterparts)
* Optimized buffered `DrawTarget` methods: `fill_contiguous`, `fill_solid`, `clear`

### Changed

* Breaking: `Command::DisplayBrightness` now takes typed `Brightness`; `brightness` is a public module and `Brightness` is re-exported at crate root
* Breaking: `init_with_addr_mode` renamed to `init`; deprecated `draw` removed
* Breaking: `DisplayConfiguration` no longer has a `DELAY` type parameter
* `embedded-graphics-core` floor raised to 0.4.1; `rust-version` set to 1.75

### Fixed

* `clear_fit_custom_stack` sends full pixel count including final partial chunk; zero chunk size errors without transport traffic
* Basic mode `fill_solid`/`fill_contiguous` clip to display bounds, preserving color alignment
* Buffered `flush` preserves dirty rectangle on transport error for retry
* Removed unchecked buffer indexing; crate contains no `unsafe` in `src/`

## [0.4.2] - 2024-10-18

### Fixed

* Fix: remove regression optimized DrawTarget
* Fix: Correct bounds check in set_pixels function for proper out-of-bounds handling

## [0.4.1] - 2024-09-29

### Added

* Add: BasicMode graphics feature, `set_pixel` and `set_pixels`
* Add: driver `Gc9a01::set_write_mode`
* Add: `Gc9a01::bounded_draw` fast bounded draw
* Add: driver `Gc9a01::draw_buffer` abstract hardware compliant
* Add: BufferedGraphics `set_pixels`
* Add `Gc9a01::clear_fit_custom_stack`

### Changed

* Deprecated `Gc9a01::draw` will be removed later

### Fixed

* Fix: embedded_graphics requirements feature
* Fix: undocumented stackoverflow case and const impl clear_fit
* Fix: Optimization embedded_graphics with BufferedGraphics
* Fix: Optimization BufferedGraphics on `fill_contiguous`
* Fix: Optimization BufferedGraphics on `fill_solid`
* Fix: Optimization BufferedGraphics on `clear`

## [0.3.1] - 2024-09-26

### Fixed

- Fix: #8 partial buffered screen update
- Fix: command, driver and basic documentation

### Changed

- Add: Brightness::brightness to access inner u8

## [0.3.0] - 2024-09-02

### Changed

- change: fn brightness::custom move from private to public

### Fixed

- clippy: allow clippy::needless_pass_by_ref_mut for driver reset
- fix: get_screen_rotation as mutable reference, but not used mutably
- clippy: src/command.rs allow clippy::doc_markdow

## [0.2.1] - 2024-04-20

### Added

- Add `cargo-husky` user hooks.
- Add `cargo fmt` inside the workflow.
- Add `just generate-readme` inside the workflow.

### Fixed

- Fix rotation memory access control (0, 90, 180, 270).

## [0.2.0] - 2024-04-19

### Changed

- __(breaking)__ Support `embedded-hal: 1.0.0` .
- __(breaking)__ Support `display-interface: 0.5.0` support. Moving `CS` to `embedded_hal::spi::SpiDevice`.

### Removed

- Move playgrounds outside the library project.

<!-- next-url -->
[unreleased]: https://github.com/IniterWorker/gc9a01/compare/0.3.1...HEAD

[0.4.1]: https://github.com/IniterWorker/gc9a01/compare/0.3.1...0.4.1
[0.3.1]: https://github.com/IniterWorker/gc9a01/compare/0.3.0...0.3.1
[0.3.0]: https://github.com/IniterWorker/gc9a01/compare/0.2.1...0.3.0
[0.2.1]: https://github.com/IniterWorker/gc9a01/compare/0.2.0...0.2.1
[0.2.0]: https://github.com/IniterWorker/gc9a01/compare/0.1.0...0.2.0