//! Buffered Graphic Implementation

use display_interface::{DataFormat, DisplayError, WriteOnlyDataCommand};
use embedded_hal::delay::DelayNs;

use crate::{display::DisplayDefinition, rotation::DisplayRotation, Gc9a01};

use super::DisplayConfiguration;

/// A mode with no additional functionality beyond that provided by the base [`Gc9a01`] struct.
#[derive(Debug, Clone)]
pub struct BasicMode;

impl BasicMode {
    /// Create a basic mode
    #[allow(clippy::missing_const_for_fn)]
    pub(crate) fn new() -> Self {
        Self {}
    }
}

impl<I, D> Gc9a01<I, D, BasicMode>
where
    I: WriteOnlyDataCommand,
    D: DisplayDefinition,
{
    /// Clear the display
    ///
    /// # Errors
    ///
    /// This method may return an error if there are communication issues with the display.
    pub fn clear(&mut self) -> Result<(), DisplayError> {
        self.set_draw_area((0, 0), self.bounds())?;
        self.set_write_mode()?;
        self.clear_fit()
    }

    /// Set the pixels directly to the hardware by setting the window from `start` to `end` based
    /// on the `Iterator<Item = u16>` provided.
    ///
    /// This function does not protect the user input.
    ///
    /// # Errors
    ///
    /// This method may return an error if there are communication issues with the display.
    pub fn set_pixels(
        &mut self,
        start: (u16, u16),
        end: (u16, u16),
        colors: &mut dyn Iterator<Item = u16>,
    ) -> Result<(), DisplayError> {
        self.set_draw_area(start, end)?;
        self.set_write_mode()?;
        self.interface.send_data(DataFormat::U16BEIter(colors))
    }

    /// Set a pixel color at `x` and `y` coordinates directly through the hardware.
    ///
    /// This function does not protect the user input.
    ///
    /// # Errors
    ///
    /// This method may return an error if there are communication issues with the display.
    pub fn set_pixel(&mut self, x: u16, y: u16, value: u16) -> Result<(), DisplayError> {
        self.set_draw_area((x, y), (x, y))?;
        self.interface.send_data(DataFormat::U16BE(&mut [value]))
    }
}

impl<I, D, DELAY> DisplayConfiguration<DELAY> for Gc9a01<I, D, BasicMode>
where
    I: WriteOnlyDataCommand,
    D: DisplayDefinition,
    DELAY: DelayNs,
{
    type Error = DisplayError;

    /// Set the display rotation.
    ///
    /// # Errors
    ///
    /// This method may return an error if there are communication issues with the display.
    fn set_rotation(&mut self, rot: DisplayRotation) -> Result<(), DisplayError> {
        self.set_display_rotation(rot)
    }

    /// Initialise and clear the display in graphics mode.
    ///
    /// # Errors
    ///
    /// This method may return an error if there are communication issues with the display.
    fn init(&mut self, delay: &mut DELAY) -> Result<(), DisplayError> {
        self.init_with_addr_mode(delay)
    }
}

#[cfg(feature = "graphics")]
use embedded_graphics_core::{
    draw_target::DrawTarget,
    geometry::Size,
    geometry::{Dimensions, OriginDimensions},
    pixelcolor::raw::RawU16,
    pixelcolor::IntoStorage,
    pixelcolor::Rgb565,
    prelude::{Point, RawData},
    primitives::Rectangle,
    Pixel,
};

#[cfg(feature = "graphics")]
impl<I, D> OriginDimensions for Gc9a01<I, D, BasicMode>
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
impl<I, D> DrawTarget for Gc9a01<I, D, BasicMode>
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
            .filter(|&Pixel(pos, _color)| bb.contains(pos))
            .try_for_each(|Pixel(pos, color)| {
                let color: RawU16 = color.into();
                let color: u16 = color.into_inner();
                #[allow(clippy::cast_sign_loss)]
                self.set_pixel(pos.x as u16, pos.y as u16, color)
            })?;
        Ok(())
    }

    fn fill_contiguous<O>(&mut self, area: &Rectangle, colors: O) -> Result<(), Self::Error>
    where
        O: IntoIterator<Item = Self::Color>,
    {
        area.bottom_right().map_or(Ok(()), |bottom_right| {
            let mut count = 0u32;
            let max = area.size.width * area.size.height;

            let mut colors = colors
                .into_iter()
                .take_while(|_| {
                    count += 1;
                    count <= max
                })
                .map(|color| RawU16::from(color).into_inner());

            #[allow(clippy::cast_sign_loss)]
            let sx = area.top_left.x as u16;
            #[allow(clippy::cast_sign_loss)]
            let sy = area.top_left.y as u16;
            #[allow(clippy::cast_sign_loss)]
            let ex = bottom_right.x as u16;
            #[allow(clippy::cast_sign_loss)]
            let ey = bottom_right.y as u16;
            self.set_pixels((sx, sy), (ex, ey), &mut colors)
        })
    }

    fn fill_solid(&mut self, area: &Rectangle, color: Self::Color) -> Result<(), Self::Error> {
        let (width, height) = self.bounds();
        let area = area.intersection(&Rectangle {
            top_left: Point::zero(),
            size: Size::new(width.into(), height.into()),
        });

        area.bottom_right().map_or(Ok(()), |bottom_right| {
            let mut count = 0u32;
            let max = area.size.width * area.size.height;

            let mut colors = core::iter::repeat(color.into_storage()).take_while(|_| {
                count += 1;
                count <= max
            });

            #[allow(clippy::cast_sign_loss)]
            let sx = area.top_left.x as u16;
            #[allow(clippy::cast_sign_loss)]
            let sy = area.top_left.y as u16;
            #[allow(clippy::cast_sign_loss)]
            let ex = bottom_right.x as u16;
            #[allow(clippy::cast_sign_loss)]
            let ey = bottom_right.y as u16;
            self.set_pixels((sx, sy), (ex, ey), &mut colors)
        })
    }

    fn clear(&mut self, color: Self::Color) -> Result<(), Self::Error> {
        let (width, height) = self.bounds();
        self.fill_solid(
            &Rectangle {
                top_left: Point::new(0, 0),
                size: Size::new(width.into(), height.into()),
            },
            color,
        )
    }
}
