//! Buffered Graphic Implementation

use display_interface::{DisplayError, WriteOnlyDataCommand};
use embedded_graphics_core::{pixelcolor::raw::RawU16, prelude::RawData};
use embedded_hal::blocking::delay::DelayMs;

use crate::{
    display::{DisplayDefinition, NewZeroed},
    rotation::DisplayRotation,
    Gc9a01,
};

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
            buffer: NewZeroed::new_zeroed(),
            min_x: u16::MAX,
            max_x: u16::MIN,
            min_y: u16::MAX,
            max_y: u16::MIN,
        }
    }
}

impl<I, D, DELAY> DisplayConfiguration<DELAY> for Gc9a01<I, D, BufferedGraphics<D>>
where
    I: WriteOnlyDataCommand,
    D: DisplayDefinition,
    DELAY: DelayMs<u8>,
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

impl<I, D> Gc9a01<I, D, BufferedGraphics<D>>
where
    I: WriteOnlyDataCommand,
    D: DisplayDefinition,
{
    /// Clear the display buffer
    /// NOTE: Must use `flush` to apply changes
    pub fn clear(&mut self) {
        for b in self.mode.buffer.as_mut() {
            *b = 0;
        }

        let (max_x, max_y) = self.dimensions();
        self.mode.min_x = u16::MIN;
        self.mode.max_x = max_x;
        self.mode.min_y = u16::MIN;
        self.mode.max_y = max_y;
    }

    pub fn fill(&mut self, color: u16) {
        for b in self.mode.buffer.as_mut() {
            *b = color;
        }

        let (max_x, max_y) = self.dimensions();
        self.mode.min_x = u16::MIN;
        self.mode.max_x = max_x;
        self.mode.min_y = u16::MIN;
        self.mode.max_y = max_y;
    }

    /// Write the display buffer
    pub fn flush(&mut self) -> Result<(), DisplayError> {
        // check if you touch anything
        if self.mode.max_x < self.mode.min_x || self.mode.max_y < self.mode.min_y {
            return Ok(());
        }

        let (width, height) = self.dimensions();

        // Determine witch bytes need to be sent
        let disp_min_x = self.mode.min_x;
        let disp_min_y = self.mode.min_y;

        let (disp_max_x, disp_max_y) = match self.display_rotation {
            DisplayRotation::Rotate0 | DisplayRotation::Rotate180 => {
                ((self.mode.max_x).min(width), (self.mode.max_y).min(height))
            }
            DisplayRotation::Rotate90 | DisplayRotation::Rotate270 => {
                ((self.mode.max_x).min(width), (self.mode.max_y).min(height))
            }
        };

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
            DisplayRotation::Rotate0 | DisplayRotation::Rotate270 => {
                self.set_draw_area(
                    (disp_min_x + offset_x, disp_min_y + D::OFFSET_Y),
                    (disp_max_x + offset_x, disp_max_y + D::OFFSET_Y),
                )?;

                Self::flush_buffer_chunks(
                    &mut self.interface,
                    self.mode.buffer.as_mut(),
                    width as usize,
                    (disp_min_x, disp_min_y),
                    (disp_max_x, disp_max_y),
                )
            }
            DisplayRotation::Rotate90 | DisplayRotation::Rotate180 => {
                self.set_draw_area(
                    (disp_min_y + offset_x, disp_min_x + D::OFFSET_Y),
                    (disp_max_y + offset_x, disp_max_x + D::OFFSET_Y),
                )?;

                Self::flush_buffer_chunks(
                    &mut self.interface,
                    self.mode.buffer.as_mut(),
                    height as usize,
                    (disp_min_y, disp_min_x),
                    (disp_max_y, disp_max_x),
                )
            }
        }
    }

    // Turn a pixel on or off
    pub fn set_pixel(&mut self, x: u32, y: u32, value: u16) {
        let value = value;
        let rotation = self.display_rotation;

        let idx = match rotation {
            DisplayRotation::Rotate0 | DisplayRotation::Rotate180 => {
                ((y as usize) * D::WIDTH as usize) + (x as usize)
            }
            DisplayRotation::Rotate90 | DisplayRotation::Rotate270 => {
                ((x as usize) * D::HEIGHT as usize) + (y as usize)
            }
        };

        if let Some(byte) = self.mode.buffer.as_mut().get_mut(idx) {
            self.mode.min_x = self.mode.min_x.min(x as u16);
            self.mode.max_x = self.mode.max_x.max(x as u16);
            self.mode.min_y = self.mode.min_y.min(y as u16);
            self.mode.max_y = self.mode.max_y.max(y as u16);

            *byte = (value >> 8) & 0xFF | (value << 8) & 0xFF00;
        }
    }
}

#[cfg(feature = "graphics")]
use embedded_graphics_core::{
    draw_target::DrawTarget,
    geometry::Size,
    geometry::{Dimensions, OriginDimensions},
    pixelcolor::Rgb565,
    Pixel,
};

use super::DisplayConfiguration;

#[cfg(feature = "graphics")]
impl<I, D> OriginDimensions for Gc9a01<I, D, BufferedGraphics<D>>
where
    I: WriteOnlyDataCommand,
    D: DisplayDefinition,
{
    fn size(&self) -> Size {
        let (w, h) = self.dimensions();
        Size::new(w.into(), h.into())
    }
}

#[cfg(feature = "graphics")]
impl<I, D> DrawTarget for Gc9a01<I, D, BufferedGraphics<D>>
where
    I: WriteOnlyDataCommand,
    D: DisplayDefinition,
{
    // TODO: figure out a way to handle all case
    type Color = Rgb565;
    type Error = DisplayError;

    fn draw_iter<O>(&mut self, pixels: O) -> Result<(), Self::Error>
    where
        O: IntoIterator<Item = Pixel<Self::Color>>,
    {
        let bb = self.bounding_box();

        pixels
            .into_iter()
            .filter(|Pixel(pos, _color)| bb.contains(*pos))
            .for_each(|Pixel(pos, color)| {
                let color: RawU16 = color.into();
                let color: u16 = color.into_inner();
                self.set_pixel(pos.x as u32, pos.y as u32, color)
            });
        Ok(())
    }
}
