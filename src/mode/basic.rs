//! Basic (direct-to-panel) mode implementation

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

impl<I, D> DisplayConfiguration for Gc9a01<I, D, BasicMode>
where
    I: WriteOnlyDataCommand,
    D: DisplayDefinition,
{
    type Error = DisplayError;

    fn set_rotation(&mut self, rot: DisplayRotation) -> Result<(), DisplayError> {
        self.set_display_rotation(rot)
    }

    #[allow(clippy::use_self)]
    fn init(&mut self, delay: &mut impl DelayNs) -> Result<(), DisplayError> {
        Gc9a01::init(self, delay)
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
fn to_u16(v: i32) -> Result<u16, DisplayError> {
    u16::try_from(v).or(Err(DisplayError::OutOfBoundsError))
}

#[cfg(feature = "graphics")]
impl<I, D> OriginDimensions for Gc9a01<I, D, BasicMode>
where
    I: WriteOnlyDataCommand,
    D: DisplayDefinition,
{
    fn size(&self) -> Size {
        let (w, h) = self.dimensions();
        Size::new(u32::from(w), u32::from(h))
    }
}

#[cfg(feature = "graphics")]
impl<I, D> DrawTarget for Gc9a01<I, D, BasicMode>
where
    I: WriteOnlyDataCommand,
    D: DisplayDefinition,
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
            .try_for_each(|Pixel(pos, color)| {
                let color: RawU16 = color.into();
                let x = to_u16(pos.x)?;
                let y = to_u16(pos.y)?;
                self.set_pixel(x, y, color.into_inner())
            })
    }

    fn fill_contiguous<O>(&mut self, area: &Rectangle, colors: O) -> Result<(), Self::Error>
    where
        O: IntoIterator<Item = Self::Color>,
    {
        let clipped = area.intersection(&self.bounding_box());
        let Some(bottom_right) = clipped.bottom_right() else {
            return Ok(());
        };
        if clipped.size.width == 0 || clipped.size.height == 0 || area.size.width == 0 {
            return Ok(());
        }
        // Offset of clipped origin inside original area; skip those colors
        // row by row so remaining colors stay aligned.
        #[allow(clippy::cast_sign_loss)]
        let dx = (clipped.top_left.x - area.top_left.x) as u32;
        #[allow(clippy::cast_sign_loss)]
        let dy = (clipped.top_left.y - area.top_left.y) as u32;
        let orig_width = area.size.width;
        let clipped_width = clipped.size.width;
        let clipped_height = clipped.size.height;
        // Position-flag iterator: keep only colors whose original
        // row/col fall inside clipped rect, preserving alignment.
        let mut src = colors.into_iter();
        let total_orig = u64::from(orig_width) * u64::from(area.size.height);
        let pos_iter = (0..total_orig).map(move |idx| {
            let row = u32::try_from(idx / u64::from(orig_width)).unwrap_or(u32::MAX);
            let col = u32::try_from(idx % u64::from(orig_width)).unwrap_or(u32::MAX);
            row >= dy && row < dy + clipped_height && col >= dx && col < dx + clipped_width
        });
        let mut visible = pos_iter.zip(src.by_ref()).filter_map(|(keep, c)| {
            if keep {
                Some(RawU16::from(c).into_inner())
            } else {
                None
            }
        });
        let sx = to_u16(clipped.top_left.x)?;
        let sy = to_u16(clipped.top_left.y)?;
        let ex = to_u16(bottom_right.x)?;
        let ey = to_u16(bottom_right.y)?;
        self.set_pixels((sx, sy), (ex, ey), &mut visible)
    }

    fn fill_solid(&mut self, area: &Rectangle, color: Self::Color) -> Result<(), Self::Error> {
        let clipped = area.intersection(&self.bounding_box());
        let Some(bottom_right) = clipped.bottom_right() else {
            return Ok(());
        };
        if clipped.size.width == 0 || clipped.size.height == 0 {
            return Ok(());
        }
        let max = clipped.size.width * clipped.size.height;
        let mut count = 0u32;
        let mut colors = core::iter::repeat(color.into_storage()).take_while(|_| {
            count += 1;
            count <= max
        });
        let sx = to_u16(clipped.top_left.x)?;
        let sy = to_u16(clipped.top_left.y)?;
        let ex = to_u16(bottom_right.x)?;
        let ey = to_u16(bottom_right.y)?;
        self.set_pixels((sx, sy), (ex, ey), &mut colors)
    }

    fn clear(&mut self, color: Self::Color) -> Result<(), Self::Error> {
        let (width, height) = self.dimensions();
        self.fill_solid(
            &Rectangle {
                top_left: Point::new(0, 0),
                size: Size::new(u32::from(width), u32::from(height)),
            },
            color,
        )
    }
}

#[cfg(all(test, feature = "graphics"))]
#[allow(clippy::unwrap_used, clippy::panic)]
mod tests {
    use crate::display::DisplayResolution240x240;
    use crate::rotation::DisplayRotation;
    use crate::test_helpers::{FakeInterface, RecordKind};
    use crate::Gc9a01;
    use embedded_graphics_core::draw_target::DrawTarget;
    use embedded_graphics_core::geometry::{Point, Size};
    use embedded_graphics_core::pixelcolor::Rgb565;
    use embedded_graphics_core::primitives::Rectangle;

    fn make() -> Gc9a01<FakeInterface, DisplayResolution240x240, super::BasicMode> {
        Gc9a01::new(
            FakeInterface::new(),
            DisplayResolution240x240,
            DisplayRotation::Rotate0,
        )
    }

    #[test]
    fn fill_solid_fullscreen_pixel_count() {
        let mut d = make();
        d.fill_solid(
            &Rectangle::new(Point::new(0, 0), Size::new(240, 240)),
            Rgb565::new(31, 0, 0),
        )
        .unwrap();
        // data: col/row params 8 bytes + 240*240*2 pixel bytes
        // (set_pixels also sends MemoryWrite command with no data)
        assert_eq!(d.interface.data_bytes_total(), 8 + 240 * 240 * 2);
    }

    #[test]
    fn fill_solid_oversized_clips_to_screen() {
        let mut d = make();
        d.fill_solid(
            &Rectangle::new(Point::new(0, 0), Size::new(300, 300)),
            Rgb565::new(0, 63, 0),
        )
        .unwrap();
        assert_eq!(d.interface.data_bytes_total(), 8 + 240 * 240 * 2);
        // draw area ends 239,239
        assert_eq!(d.interface.records[1].bytes, vec![0, 0, 0, 239]);
        assert_eq!(d.interface.records[3].bytes, vec![0, 0, 0, 239]);
    }

    #[test]
    fn fill_solid_fully_offscreen_no_traffic() {
        let mut d = make();
        d.fill_solid(
            &Rectangle::new(Point::new(500, 500), Size::new(10, 10)),
            Rgb565::new(31, 0, 0),
        )
        .unwrap();
        assert_eq!(d.interface.records.len(), 0);
    }

    #[test]
    fn fill_contiguous_negative_origin_aligns_colors() {
        let mut d = make();
        let colors: Vec<Rgb565> = (0..9u8).map(|i| Rgb565::new(i, 0, 0)).collect();
        d.fill_contiguous(&Rectangle::new(Point::new(-1, -1), Size::new(3, 3)), colors)
            .unwrap();
        // Visible pixels: orig indices 4,5,7,8 -> storage (i<<11) BE bytes
        let pixel_data: Vec<u8> = d
            .interface
            .records
            .iter()
            .filter(|r| r.kind == RecordKind::Data)
            .skip(2) // col/row params
            .flat_map(|r| r.bytes.clone())
            .collect();
        let expected: Vec<u8> = [4u16, 5, 7, 8]
            .iter()
            .flat_map(|i| (i << 11).to_be_bytes())
            .collect();
        assert_eq!(pixel_data, expected);
        // clipped draw area (0,0)-(1,1)
        assert_eq!(d.interface.records[1].bytes, vec![0, 0, 0, 1]);
        assert_eq!(d.interface.records[3].bytes, vec![0, 0, 0, 1]);
    }

    #[test]
    fn fill_contiguous_past_right_bottom_clips() {
        let mut d = make();
        // 4x4 at (238,238): visible 2x2 = orig (0,0),(1,0),(0,1),(1,1) indices 0,1,4,5
        let colors: Vec<Rgb565> = (0..16u8).map(|i| Rgb565::new(i, 0, 0)).collect();
        d.fill_contiguous(
            &Rectangle::new(Point::new(238, 238), Size::new(4, 4)),
            colors,
        )
        .unwrap();
        let pixel_data: Vec<u8> = d
            .interface
            .records
            .iter()
            .filter(|r| r.kind == RecordKind::Data)
            .skip(2)
            .flat_map(|r| r.bytes.clone())
            .collect();
        let expected: Vec<u8> = [0u16, 1, 4, 5]
            .iter()
            .flat_map(|i| (i << 11).to_be_bytes())
            .collect();
        assert_eq!(pixel_data, expected);
    }
}

/// Async basic mode: inherent async direct-to-panel methods only.
///
/// Deliberately no `DrawTarget` impl here: `DrawTarget` is a synchronous
/// trait and cannot await panel transfers. Buffered mode is the drawing
/// path for async users (draw into RAM synchronously, then async `flush()`).
#[cfg(feature = "async")]
impl<I, D> crate::Gc9a01Async<I, D, BasicMode>
where
    I: crate::interface::AsyncWriteOnlyDataCommand,
    D: DisplayDefinition,
{
    /// Clear the display.
    ///
    /// # Errors
    /// Transport errors propagate as `DisplayError`.
    pub async fn clear(&mut self) -> Result<(), DisplayError> {
        self.set_draw_area((0, 0), self.bounds()).await?;
        self.set_write_mode().await?;
        self.clear_fit().await
    }

    /// Set pixels directly to hardware from an iterator, staged through a
    /// 32-pixel stack chunk per transfer.
    ///
    /// # Errors
    /// Transport errors propagate as `DisplayError`.
    #[allow(clippy::future_not_send)]
    pub async fn set_pixels(
        &mut self,
        start: (u16, u16),
        end: (u16, u16),
        colors: &mut dyn Iterator<Item = u16>,
    ) -> Result<(), DisplayError> {
        self.set_draw_area(start, end).await?;
        self.set_write_mode().await?;
        let mut chunk = [0u16; 32];
        let mut n = 0usize;
        for color in colors {
            chunk[n] = color;
            n += 1;
            if n == chunk.len() {
                self.interface.send_pixel_data(&chunk).await?;
                n = 0;
            }
        }
        if n > 0 {
            self.interface.send_pixel_data(&chunk[..n]).await?;
        }
        Ok(())
    }

    /// Set a single pixel directly through hardware.
    ///
    /// # Errors
    /// Transport errors propagate as `DisplayError`.
    pub async fn set_pixel(&mut self, x: u16, y: u16, value: u16) -> Result<(), DisplayError> {
        self.set_draw_area((x, y), (x, y)).await?;
        self.interface.send_pixel_data(&[value]).await
    }
}

#[cfg(feature = "async")]
impl<I, D> super::AsyncDisplayConfiguration for crate::Gc9a01Async<I, D, BasicMode>
where
    I: crate::interface::AsyncWriteOnlyDataCommand,
    D: DisplayDefinition,
{
    type Error = DisplayError;

    async fn set_rotation(&mut self, rot: DisplayRotation) -> Result<(), DisplayError> {
        self.set_display_rotation(rot).await
    }

    #[allow(clippy::use_self)]
    async fn init(
        &mut self,
        delay: &mut impl embedded_hal_async::delay::DelayNs,
    ) -> Result<(), DisplayError> {
        crate::Gc9a01Async::init(self, delay).await
    }
}
