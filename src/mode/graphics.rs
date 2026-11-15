//! Buffered Graphic Implementation

use display_interface::{DisplayError, WriteOnlyDataCommand};

use crate::{
    display::{DisplayDefinition, NewZeroed},
    rotation::DisplayRotation,
    Gc9a01,
};

use super::DisplayConfiguration;

use embedded_hal::delay::DelayNs;

/// Buffered Graphic Implementation
///
/// This implementation provides a buffer in system memory.
/// This buffer is drawn to by [`set_pixel`](Gc9a01::set_pixel) commands or
/// [`embedded-graphics`](https://docs.rs/embedded-graphics) commands.
/// The display can then be updated using the [`flush`](Gc9a01::flush) method.
#[derive(Debug, Clone)]
pub struct BufferedGraphics<D, B = <D as DisplayDefinition>::Buffer>
where
    D: DisplayDefinition,
    B: AsMut<[u16]>,
{
    pub(crate) buffer: B,
    pub(crate) min_x: u16,
    pub(crate) max_x: u16,
    pub(crate) min_y: u16,
    pub(crate) max_y: u16,
    pub(crate) _phantom: core::marker::PhantomData<D>,
}

impl<D> BufferedGraphics<D, <D as DisplayDefinition>::Buffer>
where
    D: DisplayDefinition,
{
    /// Create a new buffered graphics mode instance with internal zeroed buffer.
    pub(crate) fn new() -> Self {
        let mut buffer: D::Buffer = NewZeroed::new_zeroed();
        let len = buffer.as_mut().len();
        debug_assert_eq!(len, (D::WIDTH as usize) * (D::HEIGHT as usize));
        Self {
            buffer,
            min_x: u16::MAX,
            max_x: u16::MIN,
            min_y: u16::MAX,
            max_y: u16::MIN,
            _phantom: core::marker::PhantomData,
        }
    }
}

impl<D, B> BufferedGraphics<D, B>
where
    D: DisplayDefinition,
    B: AsMut<[u16]>,
{
    /// Create buffered graphics mode with a caller-provided buffer.
    ///
    /// Buffer must hold exactly `WIDTH * HEIGHT` elements. In debug builds
    /// this is asserted; in release builds a buffer of any other length is a
    /// contract violation whose writes are bounds-checked, so worst case is
    /// clipped drawing, never undefined behaviour.
    pub fn with_buffer(mut buffer: B) -> Self {
        let len = buffer.as_mut().len();
        debug_assert_eq!(len, (D::WIDTH as usize) * (D::HEIGHT as usize));
        Self {
            buffer,
            min_x: u16::MAX,
            max_x: u16::MIN,
            min_y: u16::MAX,
            max_y: u16::MIN,
            _phantom: core::marker::PhantomData,
        }
    }

    pub(crate) fn reset_dirty(&mut self) {
        self.min_x = u16::MAX;
        self.max_x = u16::MIN;
        self.min_y = u16::MAX;
        self.max_y = u16::MIN;
    }

    fn mark_full_dirty(&mut self, rotation: DisplayRotation) {
        let (w, h) = rotated_dimensions::<D>(rotation);
        self.min_x = 0;
        self.min_y = 0;
        self.max_x = w.saturating_sub(1);
        self.max_y = h.saturating_sub(1);
    }

    pub(crate) const fn dirty(&self) -> (u16, u16, u16, u16) {
        (self.min_x, self.max_x, self.min_y, self.max_y)
    }
}

pub(crate) const fn rotated_dimensions<D: DisplayDefinition>(
    rotation: DisplayRotation,
) -> (u16, u16) {
    match rotation {
        DisplayRotation::Rotate0 | DisplayRotation::Rotate180 => (D::WIDTH, D::HEIGHT),
        DisplayRotation::Rotate90 | DisplayRotation::Rotate270 => (D::HEIGHT, D::WIDTH),
    }
}

pub(crate) const fn buffer_index(
    rotation: DisplayRotation,
    width: u16,
    height: u16,
    x: u16,
    y: u16,
) -> usize {
    match rotation {
        DisplayRotation::Rotate0 | DisplayRotation::Rotate180 => {
            (y as usize) * (width as usize) + (x as usize)
        }
        DisplayRotation::Rotate90 | DisplayRotation::Rotate270 => {
            (x as usize) * (height as usize) + (y as usize)
        }
    }
}

pub(crate) fn ram_set_pixel<D, B>(
    mode: &mut BufferedGraphics<D, B>,
    rotation: DisplayRotation,
    x: u32,
    y: u32,
    value: u16,
) where
    D: DisplayDefinition,
    B: AsMut<[u16]>,
{
    let (Ok(x16), Ok(y16)) = (u16::try_from(x), u16::try_from(y)) else {
        return;
    };
    let idx = buffer_index(rotation, D::WIDTH, D::HEIGHT, x16, y16);
    if let Some(color) = mode.buffer.as_mut().get_mut(idx) {
        mode.min_x = mode.min_x.min(x16);
        mode.max_x = mode.max_x.max(x16);
        mode.min_y = mode.min_y.min(y16);
        mode.max_y = mode.max_y.max(y16);
        *color = value;
    }
}

pub(crate) fn ram_set_pixels<D, B, T>(
    mode: &mut BufferedGraphics<D, B>,
    rotation: DisplayRotation,
    start: (u16, u16),
    end: (u16, u16),
    colors: T,
) -> Result<(), DisplayError>
where
    D: DisplayDefinition,
    B: AsMut<[u16]>,
    T: IntoIterator<Item = u16>,
{
    let start_idx = buffer_index(rotation, D::WIDTH, D::HEIGHT, start.0, start.1);
    for (i, color) in colors.into_iter().enumerate() {
        let slot = mode
            .buffer
            .as_mut()
            .get_mut(start_idx + i)
            .ok_or(DisplayError::OutOfBoundsError)?;
        *slot = color;
    }
    mode.min_x = mode.min_x.min(start.0);
    mode.max_x = mode.max_x.max(end.0);
    mode.min_y = mode.min_y.min(start.1);
    mode.max_y = mode.max_y.max(end.1);
    Ok(())
}

pub(crate) fn ram_fill_all<D, B>(
    mode: &mut BufferedGraphics<D, B>,
    rotation: DisplayRotation,
    color: u16,
) where
    D: DisplayDefinition,
    B: AsMut<[u16]>,
{
    for b in mode.buffer.as_mut() {
        *b = color;
    }
    mode.mark_full_dirty(rotation);
}

/// Computed flush geometry shared by blocking and async flush.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) struct FlushPlan {
    pub draw_start: (u16, u16),
    pub draw_end: (u16, u16),
    pub disp_width: usize,
    pub upper_left: (u16, u16),
    pub lower_right: (u16, u16),
}

pub(crate) fn flush_plan<D: DisplayDefinition>(
    rotation: DisplayRotation,
    dirty: (u16, u16, u16, u16),
) -> Option<FlushPlan> {
    let (min_x, max_x, min_y, max_y) = dirty;
    if max_x < min_x || max_y < min_y {
        return None;
    }
    let (dim_w, dim_h) = rotated_dimensions::<D>(rotation);
    let bound_width = dim_w.saturating_sub(1);
    let bound_height = dim_h.saturating_sub(1);
    let disp_min_x = min_x;
    let disp_min_y = min_y;
    let disp_max_x = max_x.min(bound_width);
    let disp_max_y = max_y.min(bound_height);
    let offset_x = match rotation {
        DisplayRotation::Rotate0 | DisplayRotation::Rotate270 => D::OFFSET_X,
        DisplayRotation::Rotate90 | DisplayRotation::Rotate180 => D::COLS - D::WIDTH - D::OFFSET_X,
    };
    match rotation {
        DisplayRotation::Rotate0 | DisplayRotation::Rotate180 => Some(FlushPlan {
            draw_start: (disp_min_x + offset_x, disp_min_y + D::OFFSET_Y),
            draw_end: (disp_max_x + offset_x, disp_max_y + D::OFFSET_Y),
            disp_width: dim_w as usize,
            upper_left: (disp_min_x, disp_min_y),
            lower_right: (disp_max_x, disp_max_y),
        }),
        DisplayRotation::Rotate90 | DisplayRotation::Rotate270 => Some(FlushPlan {
            draw_start: (disp_min_y + offset_x, disp_min_x + D::OFFSET_Y),
            draw_end: (disp_max_y + offset_x, disp_max_x + D::OFFSET_Y),
            disp_width: dim_h as usize,
            upper_left: (disp_min_y, disp_min_x),
            lower_right: (disp_max_y, disp_max_x),
        }),
    }
}

impl<I, D, B> DisplayConfiguration for Gc9a01<I, D, BufferedGraphics<D, B>>
where
    I: WriteOnlyDataCommand,
    D: DisplayDefinition,
    B: AsMut<[u16]>,
{
    type Error = DisplayError;

    fn set_rotation(&mut self, rot: DisplayRotation) -> Result<(), DisplayError> {
        self.set_display_rotation(rot)
    }

    #[allow(clippy::use_self)]
    fn init(&mut self, delay: &mut impl DelayNs) -> Result<(), DisplayError> {
        self.clear();
        Gc9a01::init(self, delay)
    }
}

impl<I, D, B> Gc9a01<I, D, BufferedGraphics<D, B>>
where
    I: WriteOnlyDataCommand,
    D: DisplayDefinition,
    B: AsMut<[u16]>,
{
    /// Clear the display buffer
    /// NOTE: Must use `flush` to apply changes
    pub fn clear(&mut self) {
        let rotation = self.display_rotation;
        ram_fill_all(&mut self.mode, rotation, 0);
    }

    pub fn fill(&mut self, color: u16) {
        let rotation = self.display_rotation;
        ram_fill_all(&mut self.mode, rotation, color);
    }

    /// Write the display buffer
    ///
    /// # Errors
    ///
    /// This method may return an error if there are communication issues with the display.
    /// On error the dirty rectangle is preserved so a retry resends it.
    pub fn flush(&mut self) -> Result<(), DisplayError> {
        let Some(plan) = flush_plan::<D>(self.display_rotation, self.mode.dirty()) else {
            return Ok(());
        };
        let result = (|| {
            self.set_draw_area(plan.draw_start, plan.draw_end)?;
            Self::flush_buffer_chunks(
                &mut self.interface,
                self.mode.buffer.as_mut(),
                plan.disp_width,
                plan.upper_left,
                plan.lower_right,
            )
        })();
        if result.is_ok() {
            self.mode.reset_dirty();
        }
        result
    }

    /// Set the pixels
    ///
    /// # Errors
    ///
    /// This method may return an out of bounds error if colors run past the buffer.
    pub fn set_pixels<T>(
        &mut self,
        start: (u16, u16),
        end: (u16, u16),
        colors: T,
    ) -> Result<(), DisplayError>
    where
        T: IntoIterator<Item = u16>,
    {
        let rotation = self.display_rotation;
        ram_set_pixels(&mut self.mode, rotation, start, end, colors)
    }

    /// Set a pixel color. If the X and Y coordinates are out of the bounds
    /// of the display, this method call is a noop
    pub fn set_pixel(&mut self, x: u32, y: u32, value: u16) {
        let rotation = self.display_rotation;
        ram_set_pixel(&mut self.mode, rotation, x, y, value);
    }
}

#[cfg(feature = "graphics")]
use embedded_graphics_core::{
    draw_target::DrawTarget,
    geometry::Size,
    geometry::{OriginDimensions, Point},
    pixelcolor::raw::RawU16,
    pixelcolor::Rgb565,
    prelude::RawData,
    primitives::Rectangle,
    Pixel,
};

#[cfg(feature = "graphics")]
fn bounding_rect<D: DisplayDefinition>(rotation: DisplayRotation) -> Rectangle {
    let (w, h) = rotated_dimensions::<D>(rotation);
    Rectangle::new(Point::zero(), Size::new(u32::from(w), u32::from(h)))
}

#[cfg(feature = "graphics")]
pub(crate) fn ram_draw_iter<D, B, O>(
    mode: &mut BufferedGraphics<D, B>,
    rotation: DisplayRotation,
    pixels: O,
) where
    D: DisplayDefinition,
    B: AsMut<[u16]>,
    O: IntoIterator<Item = Pixel<Rgb565>>,
{
    let bb = bounding_rect::<D>(rotation);
    for Pixel(pos, color) in pixels.into_iter().filter(|&Pixel(pos, _)| bb.contains(pos)) {
        let raw: RawU16 = color.into();
        #[allow(clippy::cast_sign_loss)]
        ram_set_pixel(mode, rotation, pos.x as u32, pos.y as u32, raw.into_inner());
    }
}

#[cfg(feature = "graphics")]
#[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
pub(crate) fn ram_fill_contiguous<D, B, O>(
    mode: &mut BufferedGraphics<D, B>,
    rotation: DisplayRotation,
    area: &Rectangle,
    colors: O,
) where
    D: DisplayDefinition,
    B: AsMut<[u16]>,
    O: IntoIterator<Item = Rgb565>,
{
    let clipped = area.intersection(&bounding_rect::<D>(rotation));
    if clipped.size.width == 0 || clipped.size.height == 0 {
        return;
    }
    let orig_width = area.size.width;
    if orig_width == 0 {
        return;
    }
    let dx = (clipped.top_left.x - area.top_left.x) as u32;
    let dy = (clipped.top_left.y - area.top_left.y) as u32;
    let mut it = colors.into_iter();
    // Skip leading clipped rows entirely.
    let mut to_skip = dy.saturating_mul(orig_width);
    while to_skip > 0 {
        if it.next().is_none() {
            return;
        }
        to_skip -= 1;
    }
    let trailing = orig_width - dx - clipped.size.width;
    for row in 0..clipped.size.height {
        let mut s = dx;
        while s > 0 {
            if it.next().is_none() {
                return;
            }
            s -= 1;
        }
        let y = clipped.top_left.y as u32 + row;
        for col in 0..clipped.size.width {
            let Some(color) = it.next() else { return };
            let raw: RawU16 = color.into();
            let x = clipped.top_left.x as u32 + col;
            ram_set_pixel(mode, rotation, x, y, raw.into_inner());
        }
        let mut s = trailing;
        while s > 0 {
            if it.next().is_none() {
                return;
            }
            s -= 1;
        }
    }
}

#[cfg(feature = "graphics")]
#[allow(clippy::cast_sign_loss)]
pub(crate) fn ram_fill_solid<D, B>(
    mode: &mut BufferedGraphics<D, B>,
    rotation: DisplayRotation,
    area: &Rectangle,
    color: Rgb565,
) where
    D: DisplayDefinition,
    B: AsMut<[u16]>,
{
    let clipped = area.intersection(&bounding_rect::<D>(rotation));
    if clipped.size.width == 0 || clipped.size.height == 0 {
        return;
    }
    let raw: RawU16 = color.into();
    let value = raw.into_inner();
    for row in 0..clipped.size.height {
        let y = clipped.top_left.y as u32 + row;
        for col in 0..clipped.size.width {
            let x = clipped.top_left.x as u32 + col;
            ram_set_pixel(mode, rotation, x, y, value);
        }
    }
}

#[cfg(feature = "graphics")]
impl<I, D, B> OriginDimensions for Gc9a01<I, D, BufferedGraphics<D, B>>
where
    I: WriteOnlyDataCommand,
    D: DisplayDefinition,
    B: AsMut<[u16]>,
{
    fn size(&self) -> Size {
        let (w, h) = self.dimensions();
        Size::new(u32::from(w), u32::from(h))
    }
}

#[cfg(feature = "graphics")]
impl<I, D, B> DrawTarget for Gc9a01<I, D, BufferedGraphics<D, B>>
where
    I: WriteOnlyDataCommand,
    D: DisplayDefinition,
    B: AsMut<[u16]>,
{
    type Color = Rgb565;
    type Error = DisplayError;

    fn draw_iter<O>(&mut self, pixels: O) -> Result<(), Self::Error>
    where
        O: IntoIterator<Item = Pixel<Self::Color>>,
    {
        let rotation = self.display_rotation;
        ram_draw_iter(&mut self.mode, rotation, pixels);
        Ok(())
    }

    fn fill_contiguous<O>(&mut self, area: &Rectangle, colors: O) -> Result<(), Self::Error>
    where
        O: IntoIterator<Item = Self::Color>,
    {
        let rotation = self.display_rotation;
        ram_fill_contiguous(&mut self.mode, rotation, area, colors);
        Ok(())
    }

    fn fill_solid(&mut self, area: &Rectangle, color: Self::Color) -> Result<(), Self::Error> {
        let rotation = self.display_rotation;
        ram_fill_solid(&mut self.mode, rotation, area, color);
        Ok(())
    }

    fn clear(&mut self, color: Self::Color) -> Result<(), Self::Error> {
        let raw: RawU16 = color.into();
        let rotation = self.display_rotation;
        ram_fill_all(&mut self.mode, rotation, raw.into_inner());
        Ok(())
    }
}

#[cfg(feature = "async")]
impl<I, D, B> crate::Gc9a01Async<I, D, BufferedGraphics<D, B>>
where
    I: crate::interface::AsyncWriteOnlyDataCommand,
    D: DisplayDefinition,
    B: AsMut<[u16]>,
{
    /// Clear the display buffer (RAM only). Use `flush().await` to apply.
    pub fn clear(&mut self) {
        let rotation = self.display_rotation;
        ram_fill_all(&mut self.mode, rotation, 0);
    }

    pub fn fill(&mut self, color: u16) {
        let rotation = self.display_rotation;
        ram_fill_all(&mut self.mode, rotation, color);
    }

    /// Write dirty region of buffer to panel. On error dirty rectangle is
    /// preserved so retry resends same region.
    ///
    /// # Errors
    /// Transport errors propagate as `DisplayError`.
    pub async fn flush(&mut self) -> Result<(), DisplayError> {
        let Some(plan) = flush_plan::<D>(self.display_rotation, self.mode.dirty()) else {
            return Ok(());
        };
        let result = async {
            self.set_draw_area(plan.draw_start, plan.draw_end).await?;
            Self::flush_buffer_chunks_async(
                &mut self.interface,
                self.mode.buffer.as_mut(),
                plan.disp_width,
                plan.upper_left,
                plan.lower_right,
            )
            .await
        }
        .await;
        if result.is_ok() {
            self.mode.reset_dirty();
        }
        result
    }

    /// Set pixels in RAM buffer (synchronous; no transport).
    ///
    /// # Errors
    /// Out of bounds if colors run past buffer.
    pub fn set_pixels<T>(
        &mut self,
        start: (u16, u16),
        end: (u16, u16),
        colors: T,
    ) -> Result<(), DisplayError>
    where
        T: IntoIterator<Item = u16>,
    {
        let rotation = self.display_rotation;
        ram_set_pixels(&mut self.mode, rotation, start, end, colors)
    }

    /// Set a pixel in RAM buffer (synchronous; no transport). No-op out of bounds.
    pub fn set_pixel(&mut self, x: u32, y: u32, value: u16) {
        let rotation = self.display_rotation;
        ram_set_pixel(&mut self.mode, rotation, x, y, value);
    }
}

#[cfg(feature = "async")]
impl<I, D, B> super::AsyncDisplayConfiguration for crate::Gc9a01Async<I, D, BufferedGraphics<D, B>>
where
    I: crate::interface::AsyncWriteOnlyDataCommand,
    D: DisplayDefinition,
    B: AsMut<[u16]>,
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
        self.clear();
        crate::Gc9a01Async::init(self, delay).await
    }
}

#[cfg(all(feature = "async", feature = "graphics"))]
impl<I, D, B> OriginDimensions for crate::Gc9a01Async<I, D, BufferedGraphics<D, B>>
where
    I: crate::interface::AsyncWriteOnlyDataCommand,
    D: DisplayDefinition,
    B: AsMut<[u16]>,
{
    fn size(&self) -> Size {
        let (w, h) = self.dimensions();
        Size::new(u32::from(w), u32::from(h))
    }
}

#[cfg(all(feature = "async", feature = "graphics"))]
impl<I, D, B> DrawTarget for crate::Gc9a01Async<I, D, BufferedGraphics<D, B>>
where
    I: crate::interface::AsyncWriteOnlyDataCommand,
    D: DisplayDefinition,
    B: AsMut<[u16]>,
{
    type Color = Rgb565;
    type Error = DisplayError;

    fn draw_iter<O>(&mut self, pixels: O) -> Result<(), Self::Error>
    where
        O: IntoIterator<Item = Pixel<Self::Color>>,
    {
        let rotation = self.display_rotation;
        ram_draw_iter(&mut self.mode, rotation, pixels);
        Ok(())
    }

    fn fill_contiguous<O>(&mut self, area: &Rectangle, colors: O) -> Result<(), Self::Error>
    where
        O: IntoIterator<Item = Self::Color>,
    {
        let rotation = self.display_rotation;
        ram_fill_contiguous(&mut self.mode, rotation, area, colors);
        Ok(())
    }

    fn fill_solid(&mut self, area: &Rectangle, color: Self::Color) -> Result<(), Self::Error> {
        let rotation = self.display_rotation;
        ram_fill_solid(&mut self.mode, rotation, area, color);
        Ok(())
    }

    fn clear(&mut self, color: Self::Color) -> Result<(), Self::Error> {
        let raw: RawU16 = color.into();
        let rotation = self.display_rotation;
        ram_fill_all(&mut self.mode, rotation, raw.into_inner());
        Ok(())
    }
}
#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic)]
mod tests {
    use crate::display::DisplayResolution240x240;
    use crate::rotation::DisplayRotation;
    use crate::test_helpers::FakeInterface;
    #[cfg(feature = "graphics")]
    use crate::test_helpers::RecordKind;
    use crate::Gc9a01;

    #[test]
    fn flush_preserves_dirty_on_error() {
        let mut d = Gc9a01::new(
            FakeInterface::new(),
            DisplayResolution240x240,
            DisplayRotation::Rotate0,
        )
        .into_buffered_graphics();
        d.set_pixel(1, 1, 0xF800);
        d.interface.fail_data_remaining = 1;
        assert!(d.flush().is_err());
        d.interface.clear_records();
        d.flush().unwrap();
        assert!(
            !d.interface.records.is_empty(),
            "retry should resend dirty region"
        );
    }

    #[test]
    fn clear_marks_inclusive_bounds() {
        let mut d = Gc9a01::new(
            FakeInterface::new(),
            DisplayResolution240x240,
            DisplayRotation::Rotate0,
        )
        .into_buffered_graphics();
        d.clear();
        assert_eq!((d.mode.max_x, d.mode.max_y), (239, 239));
    }

    #[cfg(feature = "graphics")]
    #[test]
    fn buffered_oversized_fill_flushes_fullscreen_red_once() {
        use embedded_graphics_core::draw_target::DrawTarget;
        use embedded_graphics_core::geometry::{Point, Size};
        use embedded_graphics_core::pixelcolor::{IntoStorage, Rgb565};
        use embedded_graphics_core::prelude::RgbColor;
        use embedded_graphics_core::primitives::Rectangle;
        let mut d = Gc9a01::new(
            FakeInterface::new(),
            DisplayResolution240x240,
            DisplayRotation::Rotate0,
        )
        .into_buffered_graphics();
        d.fill_solid(
            &Rectangle::new(Point::new(0, 0), Size::new(300, 300)),
            Rgb565::RED,
        )
        .unwrap();
        d.flush().unwrap();
        // First records: column (0,239), row (0,239), memory write, then pixel data rows.
        let recs = &d.interface.records;
        assert_eq!(recs[0].bytes, vec![0x2A]);
        assert_eq!(recs[1].bytes, vec![0, 0, 0, 239]);
        assert_eq!(recs[2].bytes, vec![0x2B]);
        assert_eq!(recs[3].bytes, vec![0, 0, 0, 239]);
        assert_eq!(recs[4].bytes, vec![0x2C]);
        let pixel_bytes: usize = recs[5..]
            .iter()
            .filter(|r| r.kind == RecordKind::Data)
            .map(|r| r.bytes.len())
            .sum();
        assert_eq!(pixel_bytes, 240 * 240 * 2);
        let red_be = Rgb565::RED.into_storage().to_be_bytes();
        for r in &recs[5..] {
            for pair in r.bytes.chunks_exact(2) {
                assert_eq!(pair, red_be);
            }
        }
        d.interface.clear_records();
        d.flush().unwrap();
        assert_eq!(d.interface.records.len(), 0);
    }

    #[cfg(feature = "graphics")]
    #[test]
    fn buffered_fill_contiguous_negative_origin_aligns_colors() {
        use embedded_graphics_core::draw_target::DrawTarget;
        use embedded_graphics_core::geometry::{Point, Size};
        use embedded_graphics_core::pixelcolor::Rgb565;
        use embedded_graphics_core::prelude::RgbColor;
        use embedded_graphics_core::primitives::Rectangle;
        // Area (-1,-1) size 3x3, colors row-major 1..=9 mapped to distinct reds.
        // Visible 2x2 should be colors at orig (1,1),(2,1),(1,2),(2,2) = indices 4,5,7,8 (0-based 4,5,7,8 -> values idx)
        let mut d = Gc9a01::new(
            FakeInterface::new(),
            DisplayResolution240x240,
            DisplayRotation::Rotate0,
        )
        .into_buffered_graphics();
        let colors: Vec<Rgb565> = (0..9u8).map(|i| Rgb565::new(i, 0, 0)).collect();
        d.fill_contiguous(&Rectangle::new(Point::new(-1, -1), Size::new(3, 3)), colors)
            .unwrap();
        // buffer indices: (0,0)=orig idx4, (1,0)=5, (0,1)=7, (1,1)=8 ; Rgb565 storage: r<<11
        let buf = d.mode.buffer.as_mut();
        assert_eq!(buf[0], 4u16 << 11);
        assert_eq!(buf[1], 5u16 << 11);
        assert_eq!(buf[240], 7u16 << 11);
        assert_eq!(buf[241], 8u16 << 11);
        let _ = Rgb565::RED;
    }
}
