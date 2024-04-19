//! SPI Display Interface

use display_interface_spi::SPIInterface;

/// SPI Interfaces for the screen
#[derive(Debug, Copy, Clone)]
pub struct SPIDisplayInterface(());

impl SPIDisplayInterface {
    #[allow(clippy::new_ret_no_self)]
    pub fn new<SPI, DC>(spi: SPI, dc: DC) -> SPIInterface<SPI, DC>
    where
        SPI: embedded_hal::spi::SpiDevice,
        DC: embedded_hal::digital::OutputPin,
    {
        SPIInterface::new(spi, dc)
    }
}
