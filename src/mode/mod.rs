mod basic;
pub use basic::*;

mod graphics;
pub use graphics::*;

use crate::rotation::DisplayRotation;
use embedded_hal::delay::DelayNs;

pub trait DisplayConfiguration<DELAY>
where
    DELAY: DelayNs,
{
    /// The type representing errors that may occur during display configuration.
    type Error;

    /// Sets the rotation of the display.
    ///
    /// # Errors
    ///
    /// This method may return an error if there are communication issues with the display.
    ///
    /// # Arguments
    ///
    /// * `rotation` - The rotation to set for the display.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the rotation is set successfully, otherwise returns an error.
    fn set_rotation(&mut self, rotation: DisplayRotation) -> Result<(), Self::Error>;

    /// Initializes and configures the display for the given mode.
    ///
    /// # Errors
    ///
    /// This method may return an error if there are communication issues with the display.
    ///
    /// # Arguments
    ///
    /// * `delay` - A mutable reference to the delay provider used for timing operations.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the display is successfully initialized and configured, otherwise returns an error.
    fn init(&mut self, delay: &mut DELAY) -> Result<(), Self::Error>;
}
