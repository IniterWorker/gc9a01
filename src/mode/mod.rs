mod basic;
pub use basic::*;

mod graphics;
pub use graphics::*;

use crate::rotation::DisplayRotation;
use embedded_hal::delay::DelayNs;

pub trait DisplayConfiguration {
    /// The type representing errors that may occur during display configuration.
    type Error;

    /// Sets the rotation of the display.
    ///
    /// # Errors
    ///
    /// This method may return an error if there are communication issues with the display.
    fn set_rotation(&mut self, rotation: DisplayRotation) -> Result<(), Self::Error>;

    /// Initializes and configures the display for the given mode.
    ///
    /// # Errors
    ///
    /// This method may return an error if there are communication issues with the display.
    fn init(&mut self, delay: &mut impl DelayNs) -> Result<(), Self::Error>;
}

#[cfg(feature = "async")]
#[allow(async_fn_in_trait)]
pub trait AsyncDisplayConfiguration {
    /// The type representing errors that may occur during display configuration.
    type Error;

    /// Sets the rotation of the display.
    ///
    /// # Errors
    ///
    /// This method may return an error if there are communication issues with the display.
    async fn set_rotation(&mut self, rotation: DisplayRotation) -> Result<(), Self::Error>;

    /// Initializes and configures the display for the given mode.
    ///
    /// # Errors
    ///
    /// This method may return an error if there are communication issues with the display.
    async fn init(
        &mut self,
        delay: &mut impl embedded_hal_async::delay::DelayNs,
    ) -> Result<(), Self::Error>;
}
