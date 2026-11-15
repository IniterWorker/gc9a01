//! # GC9A01 Display Driver
//!
//! This crate provides a driver interface to the GC9A01 LCD display driver. It
//! supports SPI via the [`display_interface`](https://docs.rs/display_interface) crate.
//!
//! The main driver is created using [`Gc9a01::new`] which accepts an interface instance, display,
//! size, rotation and mode. The following display modes are supported:
//!
//! - [`mode::BasicMode`] - A simple mode with lower level methods available.
//! - [`mode::BufferedGraphics`] - A framebuffered mode with additional methods and integration with
//!   [embedded-graphics](https://docs.rs/embedded-graphics).
//!
//! ## Support
//!
//! - [Embedded-graphics 2D graphics library](https://github.com/embedded-graphics/embedded-graphics)
//! - [Generic Interface for Display Drivers](https://github.com/therealprof/display-interface)
//! - [Embedded-hal v1.0.0](https://github.com/rust-embedded/embedded-hal/tree/embedded-hal-v1.0.0)
//!
//! ## Example
//!
//! - [Waveshare esp32-s3-touch-lcd-1-28](https://github.com/IniterWorker/esp32-s3-touch-lcd-1-28)
//!
//! ## Buffered graphics example
//!
//! ```rust
//! # fn main() {
//! # #[cfg(feature = "graphics")]
//! # {
//! use display_interface::{DataFormat, DisplayError, WriteOnlyDataCommand};
//! use embedded_graphics::{
//!     mono_font::{ascii::FONT_6X10, MonoTextStyle},
//!     pixelcolor::Rgb565,
//!     prelude::*,
//!     primitives::{PrimitiveStyle, Rectangle},
//!     text::Text,
//! };
//! use gc9a01::{display::DisplayResolution240x240, rotation::DisplayRotation, Gc9a01};
//!
//! struct Stub;
//! impl WriteOnlyDataCommand for Stub {
//!     fn send_commands(&mut self, _cmd: DataFormat<'_>) -> Result<(), DisplayError> { Ok(()) }
//!     fn send_data(&mut self, _buf: DataFormat<'_>) -> Result<(), DisplayError> { Ok(()) }
//! }
//!
//! let mut display = Gc9a01::new(Stub, DisplayResolution240x240, DisplayRotation::Rotate0)
//!     .into_buffered_graphics();
//! Rectangle::new(Point::new(10, 10), Size::new(50, 30))
//!     .into_styled(PrimitiveStyle::with_fill(Rgb565::RED))
//!     .draw(&mut display)
//!     .unwrap();
//! Text::new("Hi", Point::new(10, 60), MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE))
//!     .draw(&mut display)
//!     .unwrap();
//! display.flush().unwrap();
//! # }
//! # }
//! ```
//!
//! ## Basic mode example
//!
//! ```rust
//! use display_interface::{DataFormat, DisplayError, WriteOnlyDataCommand};
//! use gc9a01::{display::DisplayResolution240x240, rotation::DisplayRotation, Gc9a01};
//!
//! struct Stub;
//! impl WriteOnlyDataCommand for Stub {
//!     fn send_commands(&mut self, _cmd: DataFormat<'_>) -> Result<(), DisplayError> { Ok(()) }
//!     fn send_data(&mut self, _buf: DataFormat<'_>) -> Result<(), DisplayError> { Ok(()) }
//! }
//!
//! let mut display = Gc9a01::new(Stub, DisplayResolution240x240, DisplayRotation::Rotate0);
//! display.set_pixel(10, 20, 0xF800).unwrap();
//! ```
//!
//! ## Hardware
//!
//! - [1.28 inch TFT LCD Display Module Round RGB 240*240 GC9A01 Driver 4 Wire SPI](https://www.aliexpress.com/item/1005001382069930.html)

#![cfg_attr(not(test), no_std)]
// clippy warning level lints
#![warn(
    // missing_docs,
    clippy::pedantic,
    clippy::nursery,
    clippy::dbg_macro,
    clippy::unwrap_used,
    clippy::map_err_ignore,
    clippy::panic,
    clippy::unimplemented,
    clippy::unreachable,
    clippy::clone_on_ref_ptr,
    clippy::create_dir,
    clippy::exit,
    clippy::filetype_is_file,
    clippy::float_cmp_const,
    clippy::indexing_slicing,
    clippy::let_underscore_must_use,
    clippy::lossy_float_literal,
    clippy::pattern_type_mismatch,
    clippy::string_slice,
    clippy::try_err
)]
// clippy deny/error level lints, they always have  quick fix that should be preferred
#![deny(
    clippy::rc_buffer,
    clippy::rc_mutex,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::self_named_module_files,
    clippy::separated_literal_suffix,
    clippy::str_to_string,
    clippy::string_add,
    clippy::unnecessary_self_imports,
    clippy::unneeded_field_pattern,
    clippy::verbose_file_reads
)]
// allowed rules
#![allow(
    clippy::module_name_repetitions,
    clippy::similar_names,
    clippy::cast_possible_truncation,
    clippy::redundant_pub_crate,
    clippy::indexing_slicing,
    // Typestate modes intentionally split inherent impls per mode module.
    clippy::multiple_inherent_impl
)]

// export brightness
pub mod brightness;
// export commands
pub mod command;
// export screen configuration
pub mod display;
// export async transport trait
pub mod interface;
// export modes
pub mod mode;
// prelude
pub mod prelude;
// export screen rotation mode
pub mod rotation;

mod driver;
#[cfg(feature = "async")]
mod driver_async;
mod spi;

#[cfg(test)]
pub(crate) mod test_helpers;

// export the driver and interface
pub use brightness::Brightness;
pub use driver::Gc9a01;
#[cfg(feature = "async")]
pub use driver_async::Gc9a01Async;
#[cfg(feature = "async")]
pub use interface::AsyncWriteOnlyDataCommand;
#[cfg(feature = "async")]
pub use spi::AsyncSpiInterface;
pub use spi::SPIDisplayInterface;
