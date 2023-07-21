//! SPI Display Interface

use display_interface_spi::SPIInterface;

/// SPI Interfaces for the screen
#[derive(Debug, Copy, Clone)]
pub struct SPIDisplayInterface(());

impl SPIDisplayInterface {
    #[allow(clippy::new_ret_no_self)]
    pub fn new<SPI, DC, CS>(spi: SPI, dc: DC, cs: CS) -> SPIInterface<SPI, DC, CS>
    where
        SPI: embedded_hal::blocking::spi::Write<u8>,
        DC: embedded_hal::digital::v2::OutputPin,
        CS: embedded_hal::digital::v2::OutputPin,
    {
        SPIInterface::new(spi, dc, cs)
    }
}
