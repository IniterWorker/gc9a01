//! Screen Definition
//!
//! Reference all screen hardware definition

use display_interface::{DisplayError, WriteOnlyDataCommand};
use embedded_hal::delay::DelayNs;

use crate::command::{
    Command, DINVMode, Dbi, Dpi, GSMode, Gamma1, Gamma2, Gamma3, Gamma4, Logical, SSMode,
};

/// One step of a panel initialisation sequence, expressed as data so the
/// same sequence drives blocking and async configuration.
#[derive(Debug, Copy, Clone)]
pub enum InitStep {
    Cmd(Command),
    DelayMs(u32),
}

/// Screen information
///
/// This trait describes information for a particular hardware
#[allow(async_fn_in_trait)]
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

    /// Buffer type Sized. Must hold exactly `WIDTH * HEIGHT` elements.
    type Buffer: AsMut<[u16]> + NewZeroed;

    /// Panel init sequence as data. Default `configure` and (with feature
    /// `async`) default `configure_async` both run these steps in order.
    const INIT_STEPS: &[InitStep] = &[];

    /// Configuration hook to configure model-dependent configuration.
    ///
    /// Default implementation runs [`Self::INIT_STEPS`]. Custom panels may
    /// override this imperatively instead; overriding either the steps or
    /// this method is sufficient for blocking use. Async use requires the
    /// steps or a `configure_async` override: an imperative blocking-only
    /// override is not visible to the async path, which then runs the
    /// steps as given (possibly empty, i.e. no panel-specific init).
    ///
    /// # Errors
    ///
    /// This method may return an error if there are communication issues with the display.
    fn configure(
        &self,
        iface: &mut impl WriteOnlyDataCommand,
        delay: &mut impl DelayNs,
    ) -> Result<(), DisplayError> {
        for step in Self::INIT_STEPS {
            match *step {
                InitStep::Cmd(cmd) => cmd.send(iface)?,
                InitStep::DelayMs(ms) => delay.delay_ms(ms),
            }
        }
        Ok(())
    }

    /// Async counterpart of [`Self::configure`]: same steps, awaited.
    ///
    /// # Errors
    ///
    /// This method may return an error if there are communication issues with the display.
    #[cfg(feature = "async")]
    async fn configure_async(
        &self,
        iface: &mut impl crate::interface::AsyncWriteOnlyDataCommand,
        delay: &mut impl embedded_hal_async::delay::DelayNs,
    ) -> Result<(), DisplayError> {
        for step in Self::INIT_STEPS {
            match *step {
                InitStep::Cmd(cmd) => cmd.send_async(iface).await?,
                InitStep::DelayMs(ms) => delay.delay_ms(ms).await,
            }
        }
        Ok(())
    }
}

/// Screen Definition
/// Resolution 240 x 240
#[derive(Debug, Copy, Clone)]
pub struct DisplayResolution240x240;

impl DisplayDefinition for DisplayResolution240x240 {
    const WIDTH: u16 = 240;
    const HEIGHT: u16 = 240;

    type Buffer = [u16; Self::WIDTH as usize * Self::HEIGHT as usize];

    const INIT_STEPS: &[InitStep] = &[
        InitStep::Cmd(Command::InnerRegisterEnable1),
        InitStep::Cmd(Command::InnerRegisterEnable2),
        InitStep::Cmd(Command::DispalyFunctionControl(
            GSMode::G1toG32,
            SSMode::S1toS360,
            0,
            0,
        )),
        InitStep::Cmd(Command::MemoryAccessControl(
            Logical::Off,
            Logical::Off,
            Logical::Off,
            Logical::On,
            Logical::On,
            Logical::Off,
        )),
        InitStep::Cmd(Command::PixelFormatSet(Dbi::Pixel16bits, Dpi::Pixel16bits)),
        InitStep::Cmd(Command::Vreg1aVoltageControl(0x13)),
        InitStep::Cmd(Command::Vreg1bVoltageControl(0x13)),
        InitStep::Cmd(Command::Vreg2aVoltageControl(0x22)),
        InitStep::Cmd(Command::SetGamma1(Gamma1 {
            dig2j0_n: 0b1,
            vr1_n: 0b00_0101,
            dig2j1_n: 0b0,
            vr2_n: 0b00_1001,
            vr4_n: 0b1000,
            vr6_n: 0b1000,
            vr0_n: 0b10,
            vr13_n: 0b0110,
            vr20_n: 0b10_1010,
        })),
        InitStep::Cmd(Command::SetGamma2(Gamma2 {
            vr43_n: 0b100_0011,
            vr27_n: 0b11,
            vr57_n: 0b1_0000,
            vr36_n: 0b11,
            vr59_n: 0b1_0010,
            vr61_n: 0b11_0110,
            vr62_n: 0b11_0111,
            vr50_n: 0b110,
            vr63_n: 0b1111,
        })),
        InitStep::Cmd(Command::SetGamma3(Gamma3 {
            dig2j0_p: 0b1,
            vr1_p: 0b00_0101,
            dig2j1_p: 0b0,
            vr2_p: 0b00_1001,
            vr4_p: 0b1000,
            vr6_p: 0b1000,
            vr0_p: 0b10,
            vr13_p: 0b0110,
            vr20_p: 0b10_1010,
        })),
        InitStep::Cmd(Command::SetGamma4(Gamma4 {
            vr43_p: 0b100_0011,
            vr27_p: 0b11,
            vr57_p: 0b1_0000,
            vr36_p: 0b11,
            vr59_p: 0b1_0010,
            vr61_p: 0b11_0110,
            vr62_p: 0b11_0111,
            vr50_p: 0b110,
            vr63_p: 0b1111,
        })),
        InitStep::Cmd(Command::FrameRate(DINVMode::Inversion8Dot)),
        InitStep::Cmd(Command::DisplayInversion(Logical::On)),
        InitStep::Cmd(Command::SetUndocumented066h),
        InitStep::Cmd(Command::SetUndocumented067h),
        InitStep::Cmd(Command::SetUndocumented074h),
        InitStep::Cmd(Command::SetUndocumented098h),
        InitStep::Cmd(Command::TearingEffectLine(Logical::On)),
        InitStep::Cmd(Command::DisplayInversion(Logical::On)),
        InitStep::Cmd(Command::SleepMode(Logical::Off)),
        InitStep::DelayMs(120),
    ];
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

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;
    use crate::test_helpers::{FakeDelay, FakeInterface};

    pub(crate) const EXPECTED_OPCODES: &[u8] = &[
        0xFE, 0xEF, 0xB6, 0x36, 0x3A, 0xC3, 0xC4, 0xC9, 0xF0, 0xF1, 0xF2, 0xF3, 0xE8, 0x21, 0x66,
        0x67, 0x74, 0x98, 0x35, 0x21, 0x11,
    ];

    #[test]
    fn configure_opcode_sequence_and_delay() {
        let panel = DisplayResolution240x240;
        let mut iface = FakeInterface::new();
        let mut delay = FakeDelay::new();
        panel.configure(&mut iface, &mut delay).unwrap();
        assert_eq!(iface.opcodes(), EXPECTED_OPCODES);
        assert_eq!(delay.total_ms_from_ns(), 120);
        assert_eq!(delay.total_ms, 120);
    }

    #[cfg(feature = "async")]
    #[test]
    fn configure_async_same_opcodes_and_delay() {
        use crate::test_helpers::async_fakes::{FakeAsyncDelay, FakeAsyncInterface};
        use crate::test_helpers::block_on;
        let panel = DisplayResolution240x240;
        let mut iface = FakeAsyncInterface::new();
        let mut delay = FakeAsyncDelay::default();
        block_on(panel.configure_async(&mut iface, &mut delay)).unwrap();
        assert_eq!(iface.opcodes(), EXPECTED_OPCODES);
        // Full byte-for-byte parity with blocking transcript.
        let mut blocking = FakeInterface::new();
        let mut bdelay = FakeDelay::new();
        panel.configure(&mut blocking, &mut bdelay).unwrap();
        assert_eq!(iface.records, blocking.records);
        assert_eq!(delay.total_ms_from_ns(), 120);
    }
}
