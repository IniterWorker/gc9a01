//! GC9A01 Display Driver
//!
//! This Rust crate provides a driver interface to the GC9A01 RDG TFT display driver.
//! It support only SPI via the [`display_interface`](https://docs.rs/display_interface)
//! abstraction crate.
//!
//! The abstract driver must be created using [`Gc9a01::new`](crate::Gc9a01) which accepts an interface instance,
//! a const display hardware configuration, rotation and a mode.
//!
//! - [`BasicMode`](crate::mode::BasicMode) - A simple mode with lower level methods available.
//! - [`BufferedGraphics`](crate::mode::BufferedGraphics) - A framebuffered mode with additional methods and integration with
//!
//! # TODO
//! - TODO Example
//! - TODO Finish the implementation

#![cfg_attr(not(test), no_std)]

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
