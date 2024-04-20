//! Buffered Graphic Implementation

use display_interface::{DisplayError, WriteOnlyDataCommand};
use embedded_hal::delay::DelayNs;

use crate::{display::DisplayDefinition, rotation::DisplayRotation, Gc9a01};

use super::DisplayConfiguration;

/// Buffered Graphic Implementation
///
/// This implementation provides a buffer in system memory.
/// This buffer is drawn to by [`set_pixel`](Gc9a01::set_pixel) commands or
/// [`embedded-graphics`](https://docs.rs/embedded-graphics) commands.
/// The display can then be updated using the [`flush`](Gc9a01::flush) method.
#[derive(Debug, Clone)]
pub struct BasicMode;

impl BasicMode {
    /// Create a basic mode
    #[allow(clippy::missing_const_for_fn)]
    pub(crate) fn new() -> Self {
        Self {}
    }
}

impl<I, D> Gc9a01<I, D, BasicMode>
where
    I: WriteOnlyDataCommand,
    D: DisplayDefinition,
{
    /// Clear the display
    ///
    /// # Errors
    ///
    /// This method may return an error if there are communication issues with the display.
    pub fn clear(&mut self) -> Result<(), DisplayError> {
        self.set_draw_area((0, 0), self.dimensions())?;
        self.clear_fit()
    }
}

impl<I, D, DELAY> DisplayConfiguration<DELAY> for Gc9a01<I, D, BasicMode>
where
    I: WriteOnlyDataCommand,
    D: DisplayDefinition,
    DELAY: DelayNs,
{
    type Error = DisplayError;

    /// Set the display rotation.
    ///
    /// # Errors
    ///
    /// This method may return an error if there are communication issues with the display.
    fn set_rotation(&mut self, rot: DisplayRotation) -> Result<(), DisplayError> {
        self.set_display_rotation(rot)
    }

    /// Initialise and clear the display in graphics mode.
    ///
    /// # Errors
    ///
    /// This method may return an error if there are communication issues with the display.
    fn init(&mut self, delay: &mut DELAY) -> Result<(), DisplayError> {
        self.init_with_addr_mode(delay)
    }
}
