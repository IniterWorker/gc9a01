//! # GC9A01 Display Driver
//!
//! ## Modes
//!
//! - [`BasicMode`](crate::mode::BasicMode) - A simple mode with lower level methods available.
//! - [`BufferedGraphics`](crate::mode::BufferedGraphics) - A framebuffered mode with additional methods and integration with
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
    clippy::multiple_inherent_impl,
    clippy::rc_buffer,
    clippy::rc_mutex,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::self_named_module_files,
    clippy::separated_literal_suffix,
    clippy::str_to_string,
    clippy::string_add,
    clippy::string_to_string,
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
    clippy::indexing_slicing
)]

use embedded_graphics::{
    framebuffer::Framebuffer,
    pixelcolor::{
        raw::{LittleEndian, RawU16},
        Rgb565,
    },
};

// export commands
pub mod command;
// export screen configuration
pub mod display;
// export modes
pub mod mode;
// prelude
pub mod prelude;
// export screen rotation mode
pub mod rotation;

mod brightness;
mod driver;
mod spi;

// export the driver and interface
pub use driver::Gc9a01;
pub use spi::SPIDisplayInterface;

type Gc9a01Framebuffer<const WIDTH: usize, const HEIGHT: usize, const N: usize> =
    Framebuffer<Rgb565, RawU16, LittleEndian, WIDTH, HEIGHT, N>;
