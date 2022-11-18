mod basic;
pub use basic::*;

mod graphics;
pub use graphics::*;

use crate::rotation::DisplayRotation;

pub trait DisplayConfiguration {
    type Error;

    /// Set display rotation
    fn set_rotation(&mut self, rotation: DisplayRotation) -> Result<(), Self::Error>;

    /// Initialize and configure the display for the given mode
    fn init(&mut self) -> Result<(), Self::Error>;
}
