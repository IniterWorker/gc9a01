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

    /// Convert into buffered graphics mode with a caller-provided framebuffer.
    ///
    /// The driver then carries the buffer's type/lifetime (`B`), e.g.
    /// `&'a mut [u16]` or `[u16; N]`, keeping the frame out of the driver
    /// struct's own storage choice. Buffer must hold exactly
    /// `WIDTH * HEIGHT` elements (debug-asserted; see [`BufferedGraphics::with_buffer`]).
    pub fn into_buffered_graphics_with_buffer<B: AsMut<[u16]>>(
        self,
        buffer: B,
    ) -> Gc9a01<I, D, BufferedGraphics<D, B>> {
        self.into_mode(BufferedGraphics::with_buffer(buffer))
    }

    /// Convert the display into basic mode.
    pub fn into_basic_mode(self) -> Gc9a01<I, D, BasicMode> {
        self.into_mode(BasicMode::new())
    }

    /// Release the owned transport interface.
    pub fn release(self) -> I {
        self.interface
    }

    /// Initialise the screen: panel configure, rotation, default
    /// brightness, display on, then 120 ms delay.
    ///
    /// # Errors
    ///
    /// This method may return an error if there are communication issues with the display.
    pub fn init(&mut self, delay: &mut impl DelayNs) -> Result<(), DisplayError> {
        let rotation = self.display_rotation;
        self.display.configure(&mut self.interface, delay)?;
        self.set_display_rotation(rotation)?;
        self.set_brightness(Brightness::default())?;
        Command::DisplayState(Logical::On).send(&mut self.interface)?;
        delay.delay_ms(120);
        Ok(())
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
        if CLEAR_SIZE_STACK == 0 {
            return Err(DisplayError::OutOfBoundsError);
        }
        let stack_alloc = [0; CLEAR_SIZE_STACK];
        let (width, height) = self.bounds();
        let total_size = (width * height) as usize;
        let chunks = total_size.div_ceil(CLEAR_SIZE_STACK);
        let remainder = total_size % CLEAR_SIZE_STACK;
        self.set_draw_area((0, 0), (width, height))?;
        for i in 0..chunks {
            let n = if remainder != 0 && i + 1 == chunks {
                remainder
            } else {
                CLEAR_SIZE_STACK
            };
            self.interface
                .send_data(DataFormat::U16BEIter(&mut stack_alloc[..n].iter().copied()))?;
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
        }

        Ok(())
    }

    /// Change the display brightness.
    ///
    /// # Errors
    ///
    /// This method may return an error if there are communication issues with the display.
    pub fn set_brightness(&mut self, brightness: Brightness) -> Result<(), DisplayError> {
        Command::DisplayBrightness(brightness).send(&mut self.interface)
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
        for (start, len) in row_ranges(disp_width, upper_left, lower_right) {
            let row = buffer
                .get(start..start + len)
                .ok_or(DisplayError::OutOfBoundsError)?;
            interface.send_data(DataFormat::U16BEIter(&mut row.iter().copied()))?;
        }
        Ok(())
    }
}

/// Pure row-range arithmetic shared by blocking and async flush paths.
/// Yields `(start_index, len)` per row inside `upper_left..=lower_right`.
pub(crate) fn row_ranges(
    disp_width: usize,
    upper_left: (u16, u16),
    lower_right: (u16, u16),
) -> impl Iterator<Item = (usize, usize)> {
    let num_pages = if lower_right.1 >= upper_left.1 {
        (lower_right.1 - upper_left.1 + 1) as usize
    } else {
        0
    };
    let starting_page = upper_left.1 as usize;
    let page_lower = upper_left.0 as usize;
    let page_upper = ((lower_right.0 as usize) + 1).min(disp_width);
    let len = page_upper.saturating_sub(page_lower);
    (0..num_pages).map(move |r| ((starting_page + r) * disp_width + page_lower, len))
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;
    use crate::display::DisplayResolution240x240;
    use crate::test_helpers::{FakeDelay, FakeInterface, RecordKind};

    fn make() -> Gc9a01<FakeInterface, DisplayResolution240x240, BasicMode> {
        Gc9a01::new(
            FakeInterface::new(),
            DisplayResolution240x240,
            DisplayRotation::Rotate0,
        )
    }

    #[test]
    fn init_ends_display_on_and_delay() {
        let mut d = make();
        let mut delay = FakeDelay::new();
        d.init(&mut delay).unwrap();
        let ops = d.interface.opcodes();
        assert_eq!(*ops.last().unwrap(), 0x29);
        // configure 120ms + init tail 120ms
        assert_eq!(delay.total_ms_from_ns(), 240);
    }

    #[test]
    fn set_draw_area_emits_col_then_row() {
        let mut d = make();
        d.set_draw_area((1, 2), (3, 4)).unwrap();
        let got: Vec<(RecordKind, Vec<u8>)> = d
            .interface
            .records
            .iter()
            .map(|r| (r.kind, r.bytes.clone()))
            .collect();
        assert_eq!(
            got,
            vec![
                (RecordKind::Commands, vec![0x2A]),
                (RecordKind::Data, vec![0, 1, 0, 3]),
                (RecordKind::Commands, vec![0x2B]),
                (RecordKind::Data, vec![0, 2, 0, 4]),
            ]
        );
    }

    #[test]
    fn dimensions_bounds_all_rotations() {
        for rot in [
            DisplayRotation::Rotate0,
            DisplayRotation::Rotate90,
            DisplayRotation::Rotate180,
            DisplayRotation::Rotate270,
        ] {
            let mut d = make();
            d.display_rotation = rot;
            assert_eq!(d.dimensions(), (240, 240), "{rot:?}");
            assert_eq!(d.bounds(), (239, 239), "{rot:?}");
        }
    }

    #[test]
    fn clear_fit_sends_full_pixel_count() {
        let mut d = make();
        d.clear_fit().unwrap();
        // data total = col/row params (8) + pixels. bounds-based total_size = 239*239 pixels.
        let expected_pixels = 239usize * 239;
        assert_eq!(d.interface.data_bytes_total(), 8 + expected_pixels * 2);
    }

    #[test]
    fn clear_fit_zero_stack_errors_no_traffic() {
        let mut d = make();
        assert!(d.clear_fit_custom_stack::<0>().is_err());
        assert_eq!(d.interface.records.len(), 0);
    }
}
