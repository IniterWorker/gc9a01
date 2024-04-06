#![no_std]
#![no_main]

use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::{Point, RgbColor, Size},
    primitives::{Circle, Primitive, PrimitiveStyleBuilder, Rectangle},
    Drawable,
};
use esp_backtrace as _;
use gc9a01::{mode::BufferedGraphics, prelude::*, Gc9a01, SPIDisplayInterface};
use hal::{
    clock::ClockControl,
    peripherals::Peripherals,
    prelude::*,
    spi::master::Spi,
    spi::{self},
    timer::TimerGroup,
    Delay, Rtc, IO,
};

/// Test Function : will be removed later
fn draw<I: WriteOnlyDataCommand, D: DisplayDefinition>(
    display: &mut Gc9a01<I, D, BufferedGraphics<D>>,
    tick: u32,
) {
    let (w, h) = display.dimensions();
    let w = w as u32;
    let h = h as u32;
    let x = tick % w;
    let y = tick % h;

    let style = PrimitiveStyleBuilder::new()
        .stroke_width(4)
        .stroke_color(Rgb565::new(tick as u8, x as u8, y as u8))
        .fill_color(Rgb565::RED)
        .build();

    let cdiameter = 20;

    // circle
    Circle::new(
        Point::new(119 - cdiameter / 2 + 40, 119 - cdiameter / 2 + 40),
        cdiameter as u32,
    )
    .into_styled(style)
    .draw(display)
    .unwrap();

    // circle
    Circle::new(
        Point::new(119 - cdiameter / 2 - 40, 119 - cdiameter / 2 + 40),
        cdiameter as u32,
    )
    .into_styled(style)
    .draw(display)
    .unwrap();

    // rectangle
    let rw = 80;
    let rh = 20;
    Rectangle::new(
        Point::new(119 - rw / 2, 119 - rh / 2 - 40),
        Size::new(rw as u32, rh as u32),
    )
    .into_styled(style)
    .draw(display)
    .unwrap();
}

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let mut clocks = ClockControl::max(system.clock_control).freeze();
    let mut delay = Delay::new(&clocks);

    // Disable the RTC and TIMG watchdog timers
    let mut rtc = Rtc::new(peripherals.RTC_CNTL);
    let timer_group0 = TimerGroup::new(peripherals.TIMG0, &clocks);
    let mut wdt0 = timer_group0.wdt;
    let timer_group1 = TimerGroup::new(peripherals.TIMG1, &clocks);
    let mut wdt1 = timer_group1.wdt;

    rtc.rwdt.disable();
    wdt0.disable();
    wdt1.disable();

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    #[allow(unused_variables)]
    let sck = io.pins.gpio18;
    #[allow(unused_variables)]
    let mosi = io.pins.gpio23;
    #[allow(unused_variables, unused_mut)]
    let mut cs = io.pins.gpio4;
    #[allow(unused_variables, unused_mut)]
    let mut dc = io.pins.gpio2;

    let cs_output = cs.into_push_pull_output();
    let dc_output = dc.into_push_pull_output();

    let spi = Spi::new_no_cs_no_miso(
        peripherals.SPI3,
        sck,
        mosi,
        40u32.MHz(),
        spi::SpiMode::Mode0,
        &mut clocks,
    );

    let interface = SPIDisplayInterface::new(spi, dc_output, cs_output);

    let driver = Gc9a01::new(
        interface,
        DisplayResolution240x240,
        DisplayRotation::Rotate0,
    );

    let mut display = driver.into_buffered_graphics();

    display.init(&mut delay).ok();

    let mut tick: u32 = 0;
    loop {
        display.clear();
        draw(&mut display, tick);
        display.flush().ok();
        tick += 1;
    }
}
