use super::brightness::Brightness;
use super::command::{Command, Logical};
use super::display::DisplayDefinition;
use super::mode::{BasicMode, BufferedGraphics};
use super::rotation::DisplayRotation;

use display_interface::{DataFormat, DisplayError, WriteOnlyDataCommand};
use embedded_hal::blocking::delay::DelayMs;
use embedded_hal::digital::v2::OutputPin;

/// Gc9a01 Driver
pub struct Gc9a01<I, D, M>
where
    I: WriteOnlyDataCommand,
    D: DisplayDefinition,
{
    pub(crate) interface: I,
    pub(crate) display: D,
    pub(crate) mode: M,
    pub(crate) display_rotation: DisplayRotation,
}

impl<I, D, M> Gc9a01<I, D, M>
where
    I: WriteOnlyDataCommand,
    D: DisplayDefinition,
{
    /// Reset the display.
    pub fn reset<RST, DELAY>(&mut self, rst: &mut RST, delay: &mut DELAY) -> Result<(), RST::Error>
    where
        RST: OutputPin,
        DELAY: DelayMs<u8>,
    {
        fn inner_reset<RST, DELAY>(rst: &mut RST, delay: &mut DELAY) -> Result<(), RST::Error>
        where
            RST: OutputPin,
            DELAY: DelayMs<u8>,
        {
            rst.set_high()?;
            delay.delay_ms(50);
            rst.set_low()?;
            delay.delay_ms(50);
            rst.set_high()?;
            delay.delay_ms(50);
            Ok(())
        }

        inner_reset(rst, delay)
    }
}

impl<I, D> Gc9a01<I, D, BasicMode>
where
    I: WriteOnlyDataCommand,
    D: DisplayDefinition,
{
    pub fn new(interface: I, screen: D, screen_rotation: DisplayRotation) -> Self {
        Self {
            interface,
            display: screen,
            mode: BasicMode::new(),
            display_rotation: screen_rotation,
        }
    }
}

impl<I, D, M> Gc9a01<I, D, M>
where
    I: WriteOnlyDataCommand,
    D: DisplayDefinition,
{
    /// Convert the display into another interface mode.
    fn into_mode<MODE>(self, mode: MODE) -> Gc9a01<I, D, MODE> {
        Gc9a01 {
            mode,
            interface: self.interface,
            display: self.display,
            display_rotation: self.display_rotation,
        }
    }

    /// Convert the display into a buffered graphics mode, supporting
    /// [embedded-graphics](https://crates.io/crates/embedded-graphics).
    ///
    /// More information about [BufferedGraphics]
    pub fn into_buffered_graphics(self) -> Gc9a01<I, D, BufferedGraphics<D>> {
        self.into_mode(BufferedGraphics::new())
    }

    /// Initialise the screen in one of the available addressing modes.
    pub fn init_with_addr_mode(&mut self) -> Result<(), DisplayError> {
        // TODO: implement initialization sequence

        let rotation = self.display_rotation;

        // Dedicated/Custom implementation override
        self.display.configure(&mut self.interface)?;

        // Enforced context parameters
        self.set_display_rotation(rotation)?;
        self.set_brightness(Brightness::default())?;

        // Command::MemoryAddressingMode(mode).send(&mut self.interface)?;
        Command::DisplayState(Logical::On).send(&mut self.interface)?;

        Ok(())
    }

    /// Send a raw buffer to the screen.
    pub fn draw(&mut self, buffer: &[u8]) -> Result<(), DisplayError> {
        self.interface.send_data(DataFormat::U8(buffer))
    }

    /// Send a raw buffer zeroed to the screen.
    pub fn clear_fit(&mut self) -> Result<(), DisplayError> {
        self.interface
            .send_data(DataFormat::U16(&[0, D::HEIGHT * D::WIDTH]))
    }

    /// Set the screen rotation.
    pub fn set_display_rotation(&mut self, rotation: DisplayRotation) -> Result<(), DisplayError> {
        self.display_rotation = rotation;

        match self.display_rotation {
            DisplayRotation::Rotate0 => {}
            DisplayRotation::Rotate90 => {}
            DisplayRotation::Rotate180 => {}
            DisplayRotation::Rotate270 => {}
        };

        Ok(())
    }

    /// Change the display brightness.
    pub fn set_brightness(&mut self, brightness: Brightness) -> Result<(), DisplayError> {
        Command::DisplayBrightness(brightness.brightness).send(&mut self.interface)
    }

    /// Set hardware screen state
    pub fn set_screen_state(&mut self, on: Logical) -> Result<(), DisplayError> {
        Command::DisplayState(on).send(&mut self.interface)
    }

    /// Set hardware to inverse the GDDRAM framebuffer output
    pub fn set_invert_pixels(&mut self, value: bool) -> Result<(), DisplayError> {
        Command::DisplayInversion(value.into()).send(&mut self.interface)
    }

    /// Set hardware framebuffer to configure a limited area
    /// of the screen where any pixel should be draw.
    ///
    /// * (x_start, y_start) - starting point
    /// * (x_end, y_end) - ending point
    ///
    pub fn set_draw_area(
        &mut self,
        start: (u16, u16),
        end: (u16, u16),
    ) -> Result<(), DisplayError> {
        Command::ColumnAddressSet(start.0, end.0.saturating_sub(1)).send(&mut self.interface)?;
        Command::RowAddressSet(start.1, end.1.saturating_sub(1)).send(&mut self.interface)?;

        Ok(())
    }

    /// Get screen rotation
    pub fn get_screen_rotation(&mut self) -> DisplayRotation {
        self.display_rotation
    }

    /// Get pixel screen dimensions
    pub fn dimensions(&self) -> (u16, u16) {
        match self.display_rotation {
            DisplayRotation::Rotate0 | DisplayRotation::Rotate180 => (D::WIDTH, D::HEIGHT),
            DisplayRotation::Rotate90 | DisplayRotation::Rotate270 => (D::HEIGHT, D::WIDTH),
        }
    }

    /// Get pixel screen bounds (x-1, y-1)
    pub fn bounds(&self) -> (u16, u16) {
        match self.display_rotation {
            DisplayRotation::Rotate0 | DisplayRotation::Rotate180 => (D::WIDTH - 1, D::HEIGHT - 1),
            DisplayRotation::Rotate90 | DisplayRotation::Rotate270 => (D::HEIGHT - 1, D::WIDTH - 1),
        }
    }

    pub(crate) fn flush_buffer_chunks(
        interface: &mut I,
        buffer: &[u16],
        disp_width: usize,
        upper_left: (u16, u16),
        lower_right: (u16, u16),
    ) -> Result<(), DisplayError> {
        Command::MemoryWrite.send(interface)?;

        let num_pages = (lower_right.1 - upper_left.1) as usize + 1;

        let starting_page = (upper_left.1) as usize;

        // Calculate start and end X coordinates for each page
        let page_lower = upper_left.0 as usize;
        let page_upper = lower_right.0 as usize;

        // TODO: improve this
        buffer
            .chunks(disp_width)
            .skip(starting_page)
            .take(num_pages)
            .map(|s| &s[page_lower..page_upper])
            .try_for_each(|c| interface.send_data(DataFormat::U16(c)))
    }
}
