//! Crate prelude

pub use display_interface::WriteOnlyDataCommand;
pub use display_interface_spi::{SPIInterface, SPIInterfaceNoCS};

pub use super::{
    brightness::Brightness,
    display::{DisplayDefinition, DisplayResolution240x240},
    mode::DisplayConfiguration,
    rotation::DisplayRotation,
};
