//! Buffered Graphic Implementation

use core::slice;

use display_interface::{DisplayError, WriteOnlyDataCommand};
use embedded_graphics::geometry::Dimensions;
use embedded_graphics::{pixelcolor::Rgb565, prelude::*, Pixel};
use embedded_hal::delay::DelayNs;

use crate::{display::DisplayDefinition, rotation::DisplayRotation, Gc9a01, Gc9a01Framebuffer};

/// Buffered Graphic Implementation
///
/// This implementation provides a buffer in system memory.
/// This buffer is drawn to by [`set_pixel`](Gc9a01::set_pixel) commands or
/// [`embedded-graphics`](https://docs.rs/embedded-graphics) commands.
/// The display can then be updated using the [`flush`](Gc9a01::flush) method.
#[derive(Debug, Clone)]
pub struct BufferedGraphics<D>
where
    D: DisplayDefinition,
{
    buffer: D::Buffer,
    min_x: u16,
    max_x: u16,
    min_y: u16,
    max_y: u16,
}

impl<D> BufferedGraphics<D>
where
    D: DisplayDefinition,
{
    /// Create a new buffered graphics mode instance.
    pub(crate) fn new() -> Self {
        Self {
            buffer: D::new_buffer(),
            min_x: u16::MAX,
            max_x: u16::MIN,
            min_y: u16::MAX,
            max_y: u16::MIN,
        }
    }
}

impl<const WIDTH: usize, const HEIGHT: usize, const N: usize, I, D, DELAY>
    DisplayConfiguration<DELAY> for Gc9a01<I, D, BufferedGraphics<D>>
where
    I: WriteOnlyDataCommand,
    D: DisplayDefinition<Buffer = Gc9a01Framebuffer<WIDTH, HEIGHT, N>>,
    DELAY: DelayNs,
{
    type Error = DisplayError;

    /// Set display rotation
    fn set_rotation(&mut self, rot: DisplayRotation) -> Result<(), DisplayError> {
        self.set_display_rotation(rot)
    }

    /// Initialise and clear the display in graphics mode.
    fn init(&mut self, delay: &mut DELAY) -> Result<(), DisplayError> {
        self.clear();
        self.init_with_addr_mode(delay)
    }
}

impl<const WIDTH: usize, const HEIGHT: usize, const N: usize, I, D>
    Gc9a01<I, D, BufferedGraphics<D>>
where
    I: WriteOnlyDataCommand,
    D: DisplayDefinition<Buffer = Gc9a01Framebuffer<WIDTH, HEIGHT, N>>,
{
    /// Clear the display buffer
    /// NOTE: Must use `flush` to apply changes
    pub fn clear(&mut self) {
        #[allow(clippy::let_underscore_must_use)]
        let _ = self.mode.buffer.clear(Rgb565::BLACK);

        let (max_x, max_y) = self.dimensions();
        self.mode.min_x = u16::MIN;
        self.mode.max_x = max_x;
        self.mode.min_y = u16::MIN;
        self.mode.max_y = max_y;
    }

    pub fn fill(&mut self, color: Rgb565) {
        #[allow(clippy::let_underscore_must_use)]
        let _ = self.mode.buffer.clear(color);

        let (max_x, max_y) = self.dimensions();
        self.mode.min_x = u16::MIN;
        self.mode.max_x = max_x;
        self.mode.min_y = u16::MIN;
        self.mode.max_y = max_y;
    }

    #[allow(clippy::cast_ptr_alignment)]
    fn convert_u8_to_u16_slice(input: &[u8]) -> &[u16] {
        // Ensure the length is even
        assert!(input.len() % 2 == 0);

        // Convert &[u8] to &[u16] safely
        let ptr: *const u16 = input.as_ptr().cast::<u16>();
        let len = input.len() / 2;

        unsafe { slice::from_raw_parts(ptr, len) }
    }

    /// Write the display buffer
    ///
    /// # Errors
    ///
    /// This method may return an error if there are communication issues with the display.
    pub fn flush(&mut self) -> Result<(), DisplayError> {
        // check if you touch anything
        if self.mode.max_x < self.mode.min_x || self.mode.max_y < self.mode.min_y {
            return Ok(());
        }

        let (width, height) = self.dimensions();

        // Determine witch bytes need to be sent
        let disp_min_x = self.mode.min_x;
        let disp_min_y = self.mode.min_y;

        let (disp_max_x, disp_max_y) =
            ((self.mode.max_x).min(width), (self.mode.max_y).min(height));

        // reset idle state
        self.mode.min_x = u16::MAX;
        self.mode.max_x = u16::MIN;
        self.mode.min_y = u16::MAX;
        self.mode.max_y = u16::MIN;

        let offset_x = match self.display_rotation {
            DisplayRotation::Rotate0 | DisplayRotation::Rotate270 => D::OFFSET_X,
            DisplayRotation::Rotate90 | DisplayRotation::Rotate180 => {
                D::COLS - D::WIDTH - D::OFFSET_X
            }
        };

        match self.display_rotation {
            DisplayRotation::Rotate0 | DisplayRotation::Rotate180 => {
                self.set_draw_area(
                    (disp_min_x + offset_x, disp_min_y + D::OFFSET_Y),
                    (disp_max_x + offset_x, disp_max_y + D::OFFSET_Y),
                )?;

                Self::flush_buffer_chunks(
                    &mut self.interface,
                    Self::convert_u8_to_u16_slice(self.mode.buffer.data()),
                    width as usize,
                    (disp_min_x, disp_min_y),
                    (disp_max_x, disp_max_y),
                )
            }
            DisplayRotation::Rotate90 | DisplayRotation::Rotate270 => {
                self.set_draw_area(
                    (disp_min_y + offset_x, disp_min_x + D::OFFSET_Y),
                    (disp_max_y + offset_x, disp_max_x + D::OFFSET_Y),
                )?;

                Self::flush_buffer_chunks(
                    &mut self.interface,
                    Self::convert_u8_to_u16_slice(self.mode.buffer.data()),
                    height as usize,
                    (disp_min_y, disp_min_x),
                    (disp_max_y, disp_max_x),
                )
            }
        }
    }

    // Turn a pixel on or off
    pub fn set_pixel(&mut self, x: u32, y: u32, value: Rgb565) {
        let rotation = self.display_rotation;

        #[allow(clippy::cast_possible_wrap)] // for efficient
        let pos = match rotation {
            DisplayRotation::Rotate0 | DisplayRotation::Rotate180 => Point::new(x as i32, y as i32),
            DisplayRotation::Rotate90 | DisplayRotation::Rotate270 => {
                Point::new(y as i32, x as i32)
            }
        };

        self.mode.buffer.set_pixel(pos, value);

        self.mode.min_x = self.mode.min_x.min(x as u16);
        self.mode.max_x = self.mode.max_x.max(x as u16);
        self.mode.min_y = self.mode.min_y.min(y as u16);
        self.mode.max_y = self.mode.max_y.max(y as u16);
    }
}

use super::DisplayConfiguration;

impl<const WIDTH: usize, const HEIGHT: usize, const N: usize, I, D> OriginDimensions
    for Gc9a01<I, D, BufferedGraphics<D>>
where
    I: WriteOnlyDataCommand,
    D: DisplayDefinition<Buffer = Gc9a01Framebuffer<WIDTH, HEIGHT, N>>,
{
    fn size(&self) -> Size {
        let (w, h) = self.dimensions();
        Size::new(w.into(), h.into())
    }
}

impl<const WIDTH: usize, const HEIGHT: usize, const N: usize, I, D> DrawTarget
    for Gc9a01<I, D, BufferedGraphics<D>>
where
    I: WriteOnlyDataCommand,
    D: DisplayDefinition<Buffer = Gc9a01Framebuffer<WIDTH, HEIGHT, N>>,
{
    type Color = Rgb565;
    type Error = DisplayError;

    fn draw_iter<O>(&mut self, pixels: O) -> Result<(), Self::Error>
    where
        O: IntoIterator<Item = Pixel<Self::Color>>,
    {
        let bb = self.bounding_box();

        pixels
            .into_iter()
            .filter(|&Pixel(pos, _color)| bb.contains(pos))
            .for_each(|Pixel(pos, color)| {
                #[allow(clippy::cast_sign_loss)]
                self.set_pixel(pos.x as u32, pos.y as u32, color);
            });
        Ok(())
    }
}
