#![no_std]
#![no_main]

use cortex_m::delay::Delay;
use cortex_m_rt::entry;
use gc9a01::{mode::BufferedGraphics, prelude::*, Gc9a01, SPIDisplayInterface};

use embedded_graphics::{
    pixelcolor::Bgr565,
    prelude::{Point, RgbColor},
    primitives::{Circle, Primitive, PrimitiveStyleBuilder},
    Drawable,
};
use stm32_hal2::{
    clocks::Clocks,
    gpio::{OutputType, Pin, PinMode, Port},
    i2c::{I2c, I2cDevice},
    // i2c::{I2c, I2cDevice},
    pac,
    spi::{BaudRate, Spi, SpiConfig},
};

use defmt_rtt as _;
// global logger
use panic_probe as _;

/// Test Function : will be removed later
fn draw<I: WriteOnlyDataCommand, D: DisplayDefinition>(
    display: &mut Gc9a01<I, D, BufferedGraphics<D>>,
    tick: i32,
) {
    let (w, h) = display.dimensions();
    let w = w as i32;
    let h = h as i32;
    let x = tick % w;
    let y = tick % h;

    let style = PrimitiveStyleBuilder::new()
        .stroke_width(1)
        .stroke_color(Bgr565::WHITE)
        .build();

    // circle
    Circle::new(Point::new(w / 2 + x, h / 2 + y), 20)
        .into_styled(style)
        .draw(display)
        .unwrap();
}

#[entry]
fn main() -> ! {
    // Set up CPU peripherals
    #[allow(unused_variables)]
    let cp = cortex_m::Peripherals::take().unwrap();
    // Set up microcontroller peripherals
    let dp = pac::Peripherals::take().unwrap();

    defmt::debug!("Peripherals configured!");

    let clock_cfg = Clocks::default();

    clock_cfg.setup().unwrap();

    #[allow(unused_variables, unused_mut)]
    let mut delay = Delay::new(cp.SYST, clock_cfg.systick());

    // DEBUG/HARDWARE LED
    #[allow(unused_variables, unused_mut)]
    let mut led = Pin::new(Port::C, 13, PinMode::Output);

    // I2C 1
    let mut sda1 = Pin::new(Port::B, 9, PinMode::Alt(4));
    sda1.output_type(OutputType::OpenDrain);
    let mut scl1 = Pin::new(Port::B, 8, PinMode::Alt(4));
    scl1.output_type(OutputType::OpenDrain);
    #[allow(unused_variables)]
    let i2c = I2c::new(dp.I2C1, I2cDevice::One, 400_000, &clock_cfg);

    // SPI 1
    #[allow(unused_variables)]
    let sck = Pin::new(Port::A, 5, PinMode::Alt(5));
    #[allow(unused_variables)]
    let miso = Pin::new(Port::A, 6, PinMode::Alt(5));
    #[allow(unused_variables)]
    let mosi = Pin::new(Port::A, 7, PinMode::Alt(5));
    #[allow(unused_variables, unused_mut)]
    let mut cs = Pin::new(Port::A, 1, PinMode::Output);

    let spi_cfg = SpiConfig {
        mode: embedded_hal::spi::Mode {
            polarity: embedded_hal::spi::Polarity::IdleLow,
            phase: embedded_hal::spi::Phase::CaptureOnFirstTransition,
        },
        // `SpiConfig::default` is mode 0, full duplex, with software CS.
        ..Default::default()
    };

    let spi = Spi::new(dp.SPI1, spi_cfg, BaudRate::Div32);
    let interface = SPIDisplayInterface::new(spi, miso, cs);

    defmt::debug!("SPI configured!");

    let mut display_driver = Gc9a01::new(
        interface,
        DisplayResolution240x240,
        DisplayRotation::Rotate0,
    )
    .into_buffered_graphics();
    display_driver.init().ok();
    defmt::debug!("Driver configured!");

    let mut tick = 0;
    loop {
        display_driver.clear();
        draw(&mut display_driver, tick);
        display_driver.flush().ok();
        tick += 1;
    }
}

// same panicking *behavior* as `panic-probe` but doesn't print a panic message
// this prevents the panic message being printed *twice* when `defmt::panic` is invoked
#[defmt::panic_handler]
fn panic() -> ! {
    cortex_m::asm::udf()
}

/// Terminates the application and makes `probe-run` exit with exit-code = 0
pub fn exit() -> ! {
    loop {
        cortex_m::asm::bkpt();
    }
}
