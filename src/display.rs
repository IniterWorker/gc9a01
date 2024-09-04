//! Screen Definition
//!
//! Reference all screen hardware definition

use display_interface::{DisplayError, WriteOnlyDataCommand};
use embedded_graphics::{framebuffer::buffer_size, pixelcolor::Bgr565};
use embedded_hal::delay::DelayNs;

use crate::{
    command::{
        Command, DINVMode, Dbi, Dpi, GSMode, Gamma1, Gamma2, Gamma3, Gamma4, Logical, SSMode,
    },
    Gc9a01Framebuffer,
};

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

    /// Buffer data frame buffer
    type Buffer;

    /// Configuration hook to configure model-dependent configuration
    ///
    /// # Errors
    ///
    /// This method may return an error if there are communication issues with the display.
    fn configure(
        &self,
        iface: &mut impl WriteOnlyDataCommand,
        delay: &mut impl DelayNs,
    ) -> Result<(), DisplayError>;

    fn new_buffer() -> Self::Buffer;
}

/// Screen Definition
/// Resolution 240 x 240
#[derive(Debug, Copy, Clone)]
pub struct DisplayResolution240x240;

impl DisplayDefinition for DisplayResolution240x240 {
    const WIDTH: u16 = 240;
    const HEIGHT: u16 = 240;

    //type Buffer = [u16; Self::WIDTH as usize * Self::HEIGHT as usize];

    type Buffer = Gc9a01Framebuffer<
        { Self::WIDTH as usize },
        { Self::HEIGHT as usize },
        { buffer_size::<Bgr565>(Self::WIDTH as usize, Self::HEIGHT as usize) },
    >;

    fn configure(
        &self,
        iface: &mut impl WriteOnlyDataCommand,
        delay: &mut impl DelayNs,
    ) -> Result<(), DisplayError> {
        Command::InnerRegisterEnable1.send(iface)?;
        Command::InnerRegisterEnable2.send(iface)?;

        Command::DispalyFunctionControl(GSMode::G1toG32, SSMode::S1toS360, 0, 0).send(iface)?;

        Command::MemoryAccessControl(
            Logical::Off,
            Logical::Off,
            Logical::Off,
            Logical::On,
            Logical::On,
            Logical::Off,
        )
        .send(iface)?;

        // maybe an issue
        Command::PixelFormatSet(Dbi::Pixel16bits, Dpi::Pixel16bits).send(iface)?;

        // c3
        Command::Vreg1aVoltageControl(0x13).send(iface)?;
        // c4
        Command::Vreg1bVoltageControl(0x13).send(iface)?;
        // c9
        Command::Vreg2aVoltageControl(0x22).send(iface)?;

        // gamma
        Command::SetGamma1(Gamma1 {
            dig2j0_n: 0b1,
            vr1_n: 0b00_0101,
            dig2j1_n: 0b0,
            vr2_n: 0b00_1001,
            vr4_n: 0b1000,
            vr6_n: 0b1000,
            vr0_n: 0b10,
            vr13_n: 0b0110,
            vr20_n: 0b10_1010,
        })
        .send(iface)?;

        Command::SetGamma2(Gamma2 {
            vr43_n: 0b100_0011,
            vr27_n: 0b11,
            vr57_n: 0b1_0000,
            vr36_n: 0b11,
            vr59_n: 0b1_0010,
            vr61_n: 0b11_0110,
            vr62_n: 0b11_0111,
            vr50_n: 0b110,
            vr63_n: 0b1111,
        })
        .send(iface)?;

        // possible issue here
        Command::SetGamma3(Gamma3 {
            dig2j0_p: 0b1,
            vr1_p: 0b00_0101,
            dig2j1_p: 0b0,
            vr2_p: 0b00_1001,
            vr4_p: 0b1000,
            vr6_p: 0b1000,
            vr0_p: 0b10,
            vr13_p: 0b0110,
            vr20_p: 0b10_1010,
        })
        .send(iface)?;

        Command::SetGamma4(Gamma4 {
            vr43_p: 0b100_0011,
            vr27_p: 0b11,
            vr57_p: 0b1_0000,
            vr36_p: 0b11,
            vr59_p: 0b1_0010,
            vr61_p: 0b11_0110,
            vr62_p: 0b11_0111,
            vr50_p: 0b110,
            vr63_p: 0b1111,
        })
        .send(iface)?;

        // frame
        Command::FrameRate(DINVMode::Inversion8Dot).send(iface)?;

        Command::DisplayInversion(Logical::On).send(iface)?;

        // undocumented stuff here
        Command::SetUndocumented066h.send(iface)?;
        Command::SetUndocumented067h.send(iface)?;
        Command::SetUndocumented074h.send(iface)?;
        Command::SetUndocumented098h.send(iface)?;

        Command::TearingEffectLine(Logical::On).send(iface)?;
        Command::DisplayInversion(Logical::On).send(iface)?;
        Command::SleepMode(Logical::Off).send(iface)?;
        delay.delay_ms(120);

        Ok(())
    }

    fn new_buffer() -> Self::Buffer {
        Self::Buffer::new()
    }
}
