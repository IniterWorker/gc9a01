mod basic;
pub use basic::*;

mod graphics;
pub use graphics::*;

use crate::rotation::DisplayRotation;
use embedded_hal::blocking::delay::DelayMs;

pub trait DisplayConfiguration<DELAY>
where
    DELAY: DelayMs<u8>,
{
    type Error;

    /// Set display rotation
    fn set_rotation(&mut self, rotation: DisplayRotation) -> Result<(), Self::Error>;

    /// Initialize and configure the display for the given mode
    fn init(&mut self, delay: &mut DELAY) -> Result<(), Self::Error>;
}
