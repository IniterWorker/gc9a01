//! Screen Definition
//!
//! Reference all screen hardware definition

use display_interface::{DisplayError, WriteOnlyDataCommand};

use crate::command::{
    Command, DINVMode, Dbi, Dpi, GSMode, Gamma1, Gamma2, Gamma3, Gamma4, Logical,
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

    fn configure(&self, iface: &mut impl WriteOnlyDataCommand) -> Result<(), DisplayError> {
        Command::InnerRegisterEnable2.send(iface)?;

        Command::SetUndocumented0EBh(0x14).send(iface)?;

        Command::InnerRegisterEnable1.send(iface)?;
        Command::InnerRegisterEnable2.send(iface)?;

        Command::SetUndocumented0EBh(0x14).send(iface)?;
        Command::SetUndocumented084h(0x40).send(iface)?;
        Command::SetUndocumented085h(0xFF).send(iface)?;
        Command::SetUndocumented086h(0xFF).send(iface)?;
        Command::SetUndocumented087h(0xFF).send(iface)?;
        Command::SetUndocumented088h(0x0A).send(iface)?;
        Command::SetUndocumented089h(0x21).send(iface)?;
        Command::SetUndocumented08Ah(0x00).send(iface)?;
        Command::SetUndocumented08Bh(0x80).send(iface)?;
        Command::SetUndocumented08Ch(0x01).send(iface)?;
        Command::SetUndocumented08Dh(0x01).send(iface)?;
        Command::SetUndocumented08Eh(0xFF).send(iface)?;
        Command::SetUndocumented08Fh(0xFF).send(iface)?;

        Command::DispalyFunctionControl(GSMode::G1toG32, crate::command::SSMode::S1toS360, 0, 0)
            .send(iface)?;

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
        Command::PixelFormatSet(Dbi::Pixel18bits, Dpi::Pixel18bits).send(iface)?;

        Command::SetUndocumented090h.send(iface)?;
        Command::SetUndocumented0BDh.send(iface)?;
        Command::SetUndocumented0BCh.send(iface)?;
        Command::SetUndocumented0FFh.send(iface)?;

        // c3
        Command::Vreg1aVoltageControl(0x13).send(iface)?;
        // c4
        Command::Vreg1bVoltageControl(0x13).send(iface)?;
        // c9
        Command::Vreg2aVoltageControl(0x22).send(iface)?;

        Command::SetUndocumented0BEh.send(iface)?;
        Command::SetUndocumented0E1h.send(iface)?;
        Command::SetUndocumented0DFh.send(iface)?;

        // gamma
        Command::SetGamma1(Gamma1 {
            dig2j0_n: 0b1,
            vr1_n: 0b000_101,
            dig2j1_n: 0b0,
            vr2_n: 0b001_001,
            vr4_n: 0b1000,
            vr6_n: 0b1000,
            vr0_n: 0b10,
            vr13_n: 0b0110,
            vr20_n: 0b101_010,
        })
        .send(iface)?;

        Command::SetGamma2(Gamma2 {
            vr43_n: 0b1_000_011,
            vr27_n: 0b11,
            vr57_n: 0b10_000,
            vr36_n: 0b11,
            vr59_n: 0b10_010,
            vr61_n: 0b110_110,
            vr62_n: 0b110_111,
            vr50_n: 0b110,
            vr63_n: 0b1111,
        })
        .send(iface)?;

        Command::SetGamma3(Gamma3 {
            dig2j0_p: 0b1,
            vr1_p: 0b000_101,
            dig2j1_p: 0b0,
            vr2_p: 0b001_001,
            vr4_p: 0b1000,
            vr6_p: 0b1000,
            vr0_p: 0b10,
            vr13_p: 0b0110,
            vr20_p: 0b101_010,
        })
        .send(iface)?;

        Command::SetGamma4(Gamma4 {
            vr43_p: 0b1_000_011,
            vr27_p: 0b11,
            vr57_p: 0b10_000,
            vr36_p: 0b11,
            vr59_p: 0b10_010,
            vr61_p: 0b110_110,
            vr62_p: 0b110_111,
            vr50_p: 0b110,
            vr63_p: 0b1111,
        })
        .send(iface)?;

        Command::SetUndocumented0EDh.send(iface)?;
        Command::SetUndocumented0AEh.send(iface)?;
        Command::SetUndocumented0CDh.send(iface)?;
        Command::SetUndocumented070h.send(iface)?;

        // frame
        Command::FrameRate(DINVMode::Inversion8Dot).send(iface)?;

        Command::SetUndocumented062h.send(iface)?;
        Command::SetUndocumented063h.send(iface)?;
        Command::SetUndocumented064h.send(iface)?;
        Command::SetUndocumented066h.send(iface)?;
        Command::SetUndocumented067h.send(iface)?;
        Command::SetUndocumented074h.send(iface)?;
        Command::SetUndocumented098h.send(iface)?;
        // undocumented stuff here
        Command::TearingEffectLineOn(Logical::Off).send(iface)?;
        Command::SetUndocumented021h.send(iface)?;
        Command::SetUndocumented011h.send(iface)?;
        Command::SetUndocumented029h.send(iface)?;

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
