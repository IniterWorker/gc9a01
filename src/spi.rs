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

/// Async SPI transport: drives D/C pin, writes via async `SpiDevice`.
/// Chip-select is owned by the `SpiDevice`, as in the blocking path.
#[cfg(feature = "async")]
pub struct AsyncSpiInterface<SPI, DC> {
    spi: SPI,
    dc: DC,
}

#[cfg(feature = "async")]
impl<SPI, DC> AsyncSpiInterface<SPI, DC>
where
    SPI: embedded_hal_async::spi::SpiDevice,
    DC: embedded_hal::digital::OutputPin,
{
    pub const fn new(spi: SPI, dc: DC) -> Self {
        Self { spi, dc }
    }
}

#[cfg(feature = "async")]
impl<SPI, DC> crate::interface::AsyncWriteOnlyDataCommand for AsyncSpiInterface<SPI, DC>
where
    SPI: embedded_hal_async::spi::SpiDevice,
    DC: embedded_hal::digital::OutputPin,
{
    async fn send_commands(&mut self, cmd: &[u8]) -> Result<(), display_interface::DisplayError> {
        use display_interface::DisplayError;
        self.dc.set_low().or(Err(DisplayError::BusWriteError))?;
        self.spi
            .write(cmd)
            .await
            .or(Err(DisplayError::BusWriteError))
    }

    async fn send_data(&mut self, data: &[u8]) -> Result<(), display_interface::DisplayError> {
        use display_interface::DisplayError;
        self.dc.set_high().or(Err(DisplayError::BusWriteError))?;
        self.spi
            .write(data)
            .await
            .or(Err(DisplayError::BusWriteError))
    }
}
