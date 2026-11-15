//! Async driver twin of [`Gc9a01`](crate::Gc9a01), feature `async`.

use display_interface::DisplayError;
use embedded_hal::digital::OutputPin;
use embedded_hal_async::delay::DelayNs as AsyncDelayNs;

use super::brightness::Brightness;
use super::command::{Command, Logical};
use super::display::DisplayDefinition;
use super::driver::row_ranges;
use super::interface::AsyncWriteOnlyDataCommand;
use super::mode::{BasicMode, BufferedGraphics};
use super::rotation::DisplayRotation;

/// GC9A01 async driver. Method names mirror blocking [`Gc9a01`](crate::Gc9a01).
pub struct Gc9a01Async<I, D, M>
where
    I: AsyncWriteOnlyDataCommand,
    D: DisplayDefinition,
{
    pub(crate) interface: I,
    pub(crate) display: D,
    pub(crate) mode: M,
    pub(crate) display_rotation: DisplayRotation,
}

impl<I, D> Gc9a01Async<I, D, BasicMode>
where
    I: AsyncWriteOnlyDataCommand,
    D: DisplayDefinition,
{
    /// Create a basic async [`Gc9a01Async`] (construction enters via basic mode,
    /// exactly like blocking).
    pub fn new(interface: I, screen: D, screen_rotation: DisplayRotation) -> Self {
        Self {
            interface,
            display: screen,
            mode: BasicMode::new(),
            display_rotation: screen_rotation,
        }
    }
}

impl<I, D, M> Gc9a01Async<I, D, M>
where
    I: AsyncWriteOnlyDataCommand,
    D: DisplayDefinition,
{
    fn into_mode<MODE>(self, mode: MODE) -> Gc9a01Async<I, D, MODE> {
        Gc9a01Async {
            mode,
            interface: self.interface,
            display: self.display,
            display_rotation: self.display_rotation,
        }
    }

    pub fn into_buffered_graphics(self) -> Gc9a01Async<I, D, BufferedGraphics<D>> {
        self.into_mode(BufferedGraphics::new())
    }

    /// Convert into buffered graphics mode with a caller-provided framebuffer.
    /// Driver carries buffer type/lifetime, e.g. `&'a mut [u16]` or `[u16; N]`.
    pub fn into_buffered_graphics_with_buffer<B: AsMut<[u16]>>(
        self,
        buffer: B,
    ) -> Gc9a01Async<I, D, BufferedGraphics<D, B>> {
        self.into_mode(BufferedGraphics::with_buffer(buffer))
    }

    pub fn into_basic_mode(self) -> Gc9a01Async<I, D, BasicMode> {
        self.into_mode(BasicMode::new())
    }

    pub fn release(self) -> I {
        self.interface
    }

    /// Reset display. Pin ops are synchronous, delays async; same timings
    /// as blocking reset.
    ///
    /// # Errors
    /// Returns pin error on failure.
    pub async fn reset<RST, DELAY>(
        &mut self,
        rst: &mut RST,
        delay: &mut DELAY,
    ) -> Result<(), RST::Error>
    where
        RST: OutputPin,
        DELAY: AsyncDelayNs,
    {
        rst.set_high()?;
        delay.delay_ms(50).await;
        rst.set_low()?;
        delay.delay_ms(50).await;
        rst.set_high()?;
        delay.delay_ms(50).await;
        Ok(())
    }

    /// Initialise: panel configure (async), rotation, default brightness,
    /// display on, 120 ms delay.
    ///
    /// # Errors
    /// Transport errors propagate as `DisplayError`.
    pub async fn init(&mut self, delay: &mut impl AsyncDelayNs) -> Result<(), DisplayError> {
        let rotation = self.display_rotation;
        self.display
            .configure_async(&mut self.interface, delay)
            .await?;
        self.set_display_rotation(rotation).await?;
        self.set_brightness(Brightness::default()).await?;
        Command::DisplayState(Logical::On)
            .send_async(&mut self.interface)
            .await?;
        delay.delay_ms(120).await;
        Ok(())
    }

    /// # Errors
    /// Transport errors propagate as `DisplayError`.
    pub async fn draw_buffer(&mut self, buffer: &[u16]) -> Result<(), DisplayError> {
        self.interface.send_pixel_data(buffer).await
    }

    /// # Errors
    /// Transport errors propagate as `DisplayError`.
    pub async fn bounded_draw(
        &mut self,
        buffer: &[u16],
        disp_width: usize,
        upper_left: (u16, u16),
        lower_right: (u16, u16),
    ) -> Result<(), DisplayError> {
        Self::flush_buffer_chunks_async(
            &mut self.interface,
            buffer,
            disp_width,
            upper_left,
            lower_right,
        )
        .await
    }

    /// # Errors
    /// Returns `OutOfBoundsError` for zero stack size without touching
    /// transport; transport errors propagate as `DisplayError`.
    pub async fn clear_fit_custom_stack<const CLEAR_SIZE_STACK: usize>(
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
        self.set_draw_area((0, 0), (width, height)).await?;
        for i in 0..chunks {
            let n = if remainder != 0 && i + 1 == chunks {
                remainder
            } else {
                CLEAR_SIZE_STACK
            };
            self.interface.send_pixel_data(&stack_alloc[..n]).await?;
        }
        Ok(())
    }

    /// # Errors
    /// Transport errors propagate as `DisplayError`.
    pub async fn clear_fit(&mut self) -> Result<(), DisplayError> {
        self.clear_fit_custom_stack::<32>().await
    }

    /// # Errors
    /// Transport errors propagate as `DisplayError`.
    #[allow(clippy::match_same_arms)]
    pub async fn set_display_rotation(
        &mut self,
        rotation: DisplayRotation,
    ) -> Result<(), DisplayError> {
        self.display_rotation = rotation;
        let cmd = match self.display_rotation {
            DisplayRotation::Rotate0 => Command::MemoryAccessControl(
                Logical::Off,
                Logical::Off,
                Logical::Off,
                Logical::On,
                Logical::On,
                Logical::Off,
            ),
            DisplayRotation::Rotate90 => Command::MemoryAccessControl(
                Logical::On,
                Logical::Off,
                Logical::Off,
                Logical::On,
                Logical::On,
                Logical::Off,
            ),
            DisplayRotation::Rotate180 => Command::MemoryAccessControl(
                Logical::On,
                Logical::On,
                Logical::Off,
                Logical::On,
                Logical::On,
                Logical::Off,
            ),
            DisplayRotation::Rotate270 => Command::MemoryAccessControl(
                Logical::Off,
                Logical::On,
                Logical::Off,
                Logical::On,
                Logical::On,
                Logical::Off,
            ),
        };
        cmd.send_async(&mut self.interface).await
    }

    /// # Errors
    /// Transport errors propagate as `DisplayError`.
    pub async fn set_brightness(&mut self, brightness: Brightness) -> Result<(), DisplayError> {
        Command::DisplayBrightness(brightness)
            .send_async(&mut self.interface)
            .await
    }

    /// # Errors
    /// Transport errors propagate as `DisplayError`.
    pub async fn set_screen_state(&mut self, on: Logical) -> Result<(), DisplayError> {
        Command::DisplayState(on)
            .send_async(&mut self.interface)
            .await
    }

    /// # Errors
    /// Transport errors propagate as `DisplayError`.
    pub async fn set_invert_pixels(&mut self, value: bool) -> Result<(), DisplayError> {
        Command::DisplayInversion(value.into())
            .send_async(&mut self.interface)
            .await
    }

    /// # Errors
    /// Transport errors propagate as `DisplayError`.
    pub async fn set_draw_area(
        &mut self,
        start: (u16, u16),
        end: (u16, u16),
    ) -> Result<(), DisplayError> {
        Command::ColumnAddressSet(start.0, end.0)
            .send_async(&mut self.interface)
            .await?;
        Command::RowAddressSet(start.1, end.1)
            .send_async(&mut self.interface)
            .await
    }

    /// # Errors
    /// Transport errors propagate as `DisplayError`.
    pub async fn set_write_mode(&mut self) -> Result<(), DisplayError> {
        Command::MemoryWrite.send_async(&mut self.interface).await
    }

    pub const fn get_screen_rotation(&self) -> DisplayRotation {
        self.display_rotation
    }

    pub const fn dimensions(&self) -> (u16, u16) {
        match self.display_rotation {
            DisplayRotation::Rotate0 | DisplayRotation::Rotate180 => (D::WIDTH, D::HEIGHT),
            DisplayRotation::Rotate90 | DisplayRotation::Rotate270 => (D::HEIGHT, D::WIDTH),
        }
    }

    pub const fn bounds(&self) -> (u16, u16) {
        match self.display_rotation {
            DisplayRotation::Rotate0 | DisplayRotation::Rotate180 => (D::WIDTH - 1, D::HEIGHT - 1),
            DisplayRotation::Rotate90 | DisplayRotation::Rotate270 => (D::HEIGHT - 1, D::WIDTH - 1),
        }
    }

    pub(crate) async fn flush_buffer_chunks_async(
        interface: &mut I,
        buffer: &[u16],
        disp_width: usize,
        upper_left: (u16, u16),
        lower_right: (u16, u16),
    ) -> Result<(), DisplayError> {
        Command::MemoryWrite.send_async(interface).await?;
        for (start, len) in row_ranges(disp_width, upper_left, lower_right) {
            let row = buffer
                .get(start..start + len)
                .ok_or(DisplayError::OutOfBoundsError)?;
            interface.send_pixel_data(row).await?;
        }
        Ok(())
    }
}
