use super::brightness::Brightness;
use super::command::{Command, Logical};
use super::display::DisplayDefinition;
use super::mode::{BasicMode, BufferedGraphics};
use super::rotation::DisplayRotation;

use display_interface::{DataFormat, DisplayError, WriteOnlyDataCommand};
use embedded_hal::delay::DelayNs;
use embedded_hal::digital::OutputPin;

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
    ///
    /// # Errors
    ///
    /// See `OutputPin` definition for more information.
    #[allow(clippy::needless_pass_by_ref_mut)]
    pub fn reset<RST, DELAY>(&mut self, rst: &mut RST, delay: &mut DELAY) -> Result<(), RST::Error>
    where
        RST: OutputPin,
        DELAY: DelayNs,
    {
        fn inner_reset<RST, DELAY>(rst: &mut RST, delay: &mut DELAY) -> Result<(), RST::Error>
        where
            RST: OutputPin,
            DELAY: DelayNs,
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
    /// Create a basic [`Gc9a01`] interface.
    ///
    /// Use the `into_buffed_graphics` methods to enable more functionality.
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
    /// More information about [`BufferedGraphics`]
    pub fn into_buffered_graphics(self) -> Gc9a01<I, D, BufferedGraphics<D>> {
        self.into_mode(BufferedGraphics::new())
    }

    /// Initialise the screen in one of the available addressing modes.
    ///
    /// # Errors
    ///
    /// This method may return an error if there are communication issues with the display.
    pub fn init_with_addr_mode(&mut self, delay: &mut impl DelayNs) -> Result<(), DisplayError> {
        // TODO: implement initialization sequence

        let rotation = self.display_rotation;

        // Dedicated/Custom implementation override
        self.display.configure(&mut self.interface, delay)?;

        // Enforced context parameters
        self.set_display_rotation(rotation)?;
        self.set_brightness(Brightness::default())?;

        // Command::MemoryAddressingMode(mode).send(&mut self.interface)?;
        Command::DisplayState(Logical::On).send(&mut self.interface)?;
        delay.delay_ms(120);

        Ok(())
    }

    /// Send a raw buffer to the screen.
    ///
    /// # Errors
    ///
    /// This method may return an error if there are communication issues with the display.
    #[deprecated(note = "Use `draw_buffer` instead")]
    pub fn draw(&mut self, buffer: &[u8]) -> Result<(), DisplayError> {
        self.interface.send_data(DataFormat::U8(buffer))
    }

    /// Send a raw buffer to the screen.
    ///
    /// # Errors
    ///
    /// This method may return an error if there are communication issues with the display.
    pub fn draw_buffer(&mut self, buffer: &[u16]) -> Result<(), DisplayError> {
        self.interface
            .send_data(DataFormat::U16BEIter(&mut buffer.iter().copied()))
    }

    /// Send the data to the display for drawing at the current position in the framebuffer
    /// and advance the position accordingly. Ref. `set_draw_area` to modify the affected area by
    /// this method.
    ///
    /// # Notes
    ///
    /// This method takes advantage of the bounding box for faster writes. Meaning, it will
    /// split into chuncks of write operations.
    ///
    /// # Errors
    ///
    /// This method may return an error if there are communication issues with the display.
    pub fn bounded_draw(
        &mut self,
        buffer: &[u16],
        disp_width: usize,
        upper_left: (u16, u16),
        lower_right: (u16, u16),
    ) -> Result<(), DisplayError> {
        Self::flush_buffer_chunks(
            &mut self.interface,
            buffer,
            disp_width,
            upper_left,
            lower_right,
        )
    }

    /// Clears the screen by sending a zeroed buffer using a custom stack size for allocation.
    ///
    /// This function uses a stack-allocated buffer of size `CLEAR_SIZE_STACK` to send
    /// a portion of the screen-clearing data to the display. The full display area is divided
    /// into smaller chunks of size `CLEAR_SIZE_STACK`, which are then sent iteratively to avoid
    /// allocating large amounts of memory on the stack in one go.
    ///
    /// This function uses `set_draw_area`.
    ///
    /// # Type Parameters
    ///
    /// - `CLEAR_SIZE_STACK`: The size of the stack-allocated buffer used to send each chunk of data
    ///   to the display. This value determines the number of zeroed `u16` values sent per iteration.
    ///
    /// # Errors
    ///
    /// This method returns an error if there are communication issues while sending the data
    /// to the display.
    ///
    /// # Panics
    ///
    /// This method will stackoverflow if the value of `CLEAR_SIZE_STACK` exceeds the remaining available stack space.
    pub fn clear_fit_custom_stack<const CLEAR_SIZE_STACK: usize>(
        &mut self,
    ) -> Result<(), DisplayError> {
        // Allocate a zeroed buffer on the stack
        let stack_alloc = [0; CLEAR_SIZE_STACK];

        // Get the width and height of the display
        let (width, height) = self.bounds();
        let total_size = (width * height) as usize;

        // Calculate how many chunks of size CLEAR_SIZE_STACK are needed
        let mut total_it = total_size / CLEAR_SIZE_STACK;

        // Set the draw area to the entire screen
        self.set_draw_area((0, 0), (width, height))?;

        // Send the zeroed buffer in chunks until the entire screen is cleared
        while total_it > 1 {
            self.interface
                .send_data(DataFormat::U16BEIter(&mut stack_alloc.iter().copied()))?;
            total_it -= 1;
        }

        Ok(())
    }

    /// Clears the screen by sending a zeroed buffer using a default stack size.
    ///
    /// This method sends zeroed `u16` values to the display in chunks to clear the entire screen.
    /// It uses a default stack allocation size of 32 `u16` values (64 bytes) per iteration,
    /// making it suitable for most small embedded contexts where stack space is limited.
    ///
    /// If you need to customize the size of the stack-allocated buffer, use
    /// [`clear_fit_custom_stack`] with a specific stack size.
    ///
    /// This function uses `set_draw_area`.
    ///
    /// # Errors
    ///
    /// This method returns an error if there are communication issues while sending the data
    /// to the display.
    ///
    /// # Notes
    ///
    /// The default stack allocation size is 64 bytes (32 `u16` values).
    pub fn clear_fit(&mut self) -> Result<(), DisplayError> {
        self.clear_fit_custom_stack::<32>()
    }

    /// Set the screen rotation.
    ///
    /// # Errors
    ///
    /// This method may return an error if there are communication issues with the display.
    #[allow(clippy::match_same_arms)]
    pub fn set_display_rotation(&mut self, rotation: DisplayRotation) -> Result<(), DisplayError> {
        self.display_rotation = rotation;

        match self.display_rotation {
            DisplayRotation::Rotate0 => Command::MemoryAccessControl(
                Logical::Off,
                Logical::Off,
                Logical::Off,
                Logical::On,
                Logical::On,
                Logical::Off,
            )
            .send(&mut self.interface)?,
            DisplayRotation::Rotate90 => Command::MemoryAccessControl(
                Logical::On,
                Logical::Off,
                Logical::Off,
                Logical::On,
                Logical::On,
                Logical::Off,
            )
            .send(&mut self.interface)?,
            DisplayRotation::Rotate180 => Command::MemoryAccessControl(
                Logical::On,
                Logical::On,
                Logical::Off,
                Logical::On,
                Logical::On,
                Logical::Off,
            )
            .send(&mut self.interface)?,
            DisplayRotation::Rotate270 => Command::MemoryAccessControl(
                Logical::Off,
                Logical::On,
                Logical::Off,
                Logical::On,
                Logical::On,
                Logical::Off,
            )
            .send(&mut self.interface)?,
        };

        Ok(())
    }

    /// Change the display brightness.
    ///
    /// # Errors
    ///
    /// This method may return an error if there are communication issues with the display.
    pub fn set_brightness(&mut self, brightness: Brightness) -> Result<(), DisplayError> {
        Command::DisplayBrightness(brightness.brightness).send(&mut self.interface)
    }

    /// Set hardware screen state
    ///
    /// # Errors
    ///
    /// This method may return an error if there are communication issues with the display.
    pub fn set_screen_state(&mut self, on: Logical) -> Result<(), DisplayError> {
        Command::DisplayState(on).send(&mut self.interface)
    }

    /// Set hardware to inverse the GDDRAM framebuffer output
    ///
    /// # Errors
    ///
    /// This method may return an error if there are communication issues with the display.
    pub fn set_invert_pixels(&mut self, value: bool) -> Result<(), DisplayError> {
        Command::DisplayInversion(value.into()).send(&mut self.interface)
    }

    /// Set hardware framebuffer to configure a limited area
    /// of the screen where any pixel should be draw.
    ///
    /// * (`x_start`, `y_start`) - starting point
    /// * (`x_end`, `y_end`) - ending point
    ///
    /// # Errors
    ///
    /// This method may return an error if there are communication issues with the display.
    pub fn set_draw_area(
        &mut self,
        start: (u16, u16),
        end: (u16, u16),
    ) -> Result<(), DisplayError> {
        Command::ColumnAddressSet(start.0, end.0).send(&mut self.interface)?;
        Command::RowAddressSet(start.1, end.1).send(&mut self.interface)?;

        Ok(())
    }

    /// Set the hardware framebuffer to await incoming colors
    ///
    /// # Errors
    ///
    /// This method may return an error if there are communication issues with the display.
    pub fn set_write_mode(&mut self) -> Result<(), DisplayError> {
        Command::MemoryWrite.send(&mut self.interface)?;
        Ok(())
    }

    /// Get screen rotation
    pub const fn get_screen_rotation(&self) -> DisplayRotation {
        self.display_rotation
    }

    /// Get pixel screen dimensions
    pub const fn dimensions(&self) -> (u16, u16) {
        match self.display_rotation {
            DisplayRotation::Rotate0 | DisplayRotation::Rotate180 => (D::WIDTH, D::HEIGHT),
            DisplayRotation::Rotate90 | DisplayRotation::Rotate270 => (D::HEIGHT, D::WIDTH),
        }
    }

    /// Get pixel screen bounds (x-1, y-1)
    pub const fn bounds(&self) -> (u16, u16) {
        match self.display_rotation {
            DisplayRotation::Rotate0 | DisplayRotation::Rotate180 => (D::WIDTH - 1, D::HEIGHT - 1),
            DisplayRotation::Rotate90 | DisplayRotation::Rotate270 => (D::HEIGHT - 1, D::WIDTH - 1),
        }
    }

    /// Flush the buffer by chuncks
    ///
    /// # Errors
    ///
    /// This method may return an error if there are communication issues with the display.
    pub(crate) fn flush_buffer_chunks(
        interface: &mut I,
        buffer: &[u16],
        disp_width: usize,
        upper_left: (u16, u16),
        lower_right: (u16, u16),
    ) -> Result<(), DisplayError> {
        Command::MemoryWrite.send(interface)?;

        // Number of rows to process (Y range)
        let num_pages = (lower_right.1 - upper_left.1 + 1) as usize;

        // Starting row (Y coordinate)
        let starting_page = upper_left.1 as usize;

        // X coordinates (columns) for the rectangle
        let page_lower = upper_left.0 as usize;
        let page_upper = ((lower_right.0 + 1) as usize).min(disp_width); // +1 to include the last column

        // Process the buffer in rows (chunks of disp_width)
        buffer
            .chunks(disp_width)
            .skip(starting_page)
            .take(num_pages)
            .map(|s| &s[page_lower..page_upper])
            .try_for_each(|c| interface.send_data(DataFormat::U16BEIter(&mut c.iter().copied())))
    }
}
