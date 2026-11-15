//! Compile-only blocking buffered example over stub transport.
//! Sequence: new -> into_buffered_graphics -> init -> draw -> flush.

use display_interface::{DataFormat, DisplayError, WriteOnlyDataCommand};
use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
};
use embedded_hal::delay::DelayNs;
use gc9a01::{display::DisplayResolution240x240, rotation::DisplayRotation, Gc9a01};

struct StubInterface;

impl WriteOnlyDataCommand for StubInterface {
    fn send_commands(&mut self, _cmd: DataFormat<'_>) -> Result<(), DisplayError> {
        Ok(())
    }
    fn send_data(&mut self, _buf: DataFormat<'_>) -> Result<(), DisplayError> {
        Ok(())
    }
}

struct StubDelay;

impl DelayNs for StubDelay {
    fn delay_ns(&mut self, _ns: u32) {}
}

fn main() {
    let mut display = Gc9a01::new(
        StubInterface,
        DisplayResolution240x240,
        DisplayRotation::Rotate0,
    )
    .into_buffered_graphics();
    display.init(&mut StubDelay).unwrap();
    Rectangle::new(Point::new(20, 20), Size::new(80, 60))
        .into_styled(PrimitiveStyle::with_fill(Rgb565::RED))
        .draw(&mut display)
        .unwrap();
    display.flush().unwrap();
}
