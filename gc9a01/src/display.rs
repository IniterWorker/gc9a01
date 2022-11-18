//! Screen Definition
//!
//! Reference all screen hardware definition

use display_interface::{DisplayError, WriteOnlyDataCommand};

/// Screen information
///
/// This trait describes information for a particular hardware
pub trait DisplayDefinition {
    /// The screen/pixels maximum width (X)
    const WIDTH: u16;

    /// The screen/pixels maximum height (Y)
    const HEIGHT: u16;

    /// The screen X offset
    const OFFSET_X: u16 = 0;

    /// The screen Y offset
    const OFFSET_Y: u16 = 0;

    /// The driver maximum cols
    const COLS: u16 = 240;

    /// The driver maximum rows    
    const ROWS: u16 = 240;

    /// Buffer type Sized
    type Buffer: AsMut<[u16]> + NewZeroed;

    /// Configuration hook to configure model-dependent configuration
    fn configure(&self, iface: &mut impl WriteOnlyDataCommand) -> Result<(), DisplayError>;
}

/// Screen Definition
/// Resolution 240 x 240
#[derive(Debug, Copy, Clone)]
pub struct DisplayResolution240x240;

impl DisplayDefinition for DisplayResolution240x240 {
    const WIDTH: u16 = 240;
    const HEIGHT: u16 = 240;

    type Buffer = [u16; Self::WIDTH as usize * Self::HEIGHT as usize];

    fn configure(&self, _iface: &mut impl WriteOnlyDataCommand) -> Result<(), DisplayError> {
        Ok(())
    }
}

pub trait NewZeroed {
    /// Creates a new value with its memory set to zero
    fn new_zeroed() -> Self;
}

impl<const N: usize> NewZeroed for [u16; N] {
    fn new_zeroed() -> Self {
        [0u16; N]
    }
}
