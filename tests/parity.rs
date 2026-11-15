//! Blocking vs async parity: identical transcripts and delay totals for
//! the same scripted sequence.
#![allow(clippy::unwrap_used)]

use display_interface::{DataFormat, DisplayError, WriteOnlyDataCommand};
use embedded_hal::delay::DelayNs as BlockingDelayNs;
use gc9a01::display::DisplayResolution240x240;
use gc9a01::rotation::DisplayRotation;
use gc9a01::{Brightness, Gc9a01};
#[cfg(all(feature = "async", feature = "graphics"))]
use std::future::Future;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Commands,
    Data,
}

type Transcript = Vec<(Kind, Vec<u8>)>;

#[derive(Default)]
struct BlockingFake {
    records: Transcript,
}

fn expand(fmt: DataFormat<'_>) -> Vec<u8> {
    match fmt {
        DataFormat::U8(s) => s.to_vec(),
        DataFormat::U16(s) => s.iter().flat_map(|v| v.to_ne_bytes()).collect(),
        DataFormat::U16BE(s) => s.iter().flat_map(|v| v.to_be_bytes()).collect(),
        DataFormat::U16LE(s) => s.iter().flat_map(|v| v.to_le_bytes()).collect(),
        DataFormat::U8Iter(it) => it.collect(),
        DataFormat::U16BEIter(it) => it.flat_map(u16::to_be_bytes).collect(),
        DataFormat::U16LEIter(it) => it.flat_map(u16::to_le_bytes).collect(),
        _ => panic!("unsupported format"),
    }
}

impl WriteOnlyDataCommand for BlockingFake {
    fn send_commands(&mut self, cmd: DataFormat<'_>) -> Result<(), DisplayError> {
        self.records.push((Kind::Commands, expand(cmd)));
        Ok(())
    }
    fn send_data(&mut self, buf: DataFormat<'_>) -> Result<(), DisplayError> {
        self.records.push((Kind::Data, expand(buf)));
        Ok(())
    }
}

#[derive(Default)]
struct BlockingDelay {
    total_ns: u64,
}

impl BlockingDelayNs for BlockingDelay {
    fn delay_ns(&mut self, ns: u32) {
        self.total_ns += u64::from(ns);
    }
    fn delay_us(&mut self, us: u32) {
        self.total_ns += u64::from(us) * 1_000;
    }
    fn delay_ms(&mut self, ms: u32) {
        self.total_ns += u64::from(ms) * 1_000_000;
    }
}

#[cfg(all(feature = "async", feature = "graphics"))]
fn block_on<F: Future>(fut: F) -> F::Output {
    use std::sync::Arc;
    use std::task::{Context, Poll, Wake, Waker};
    struct NoopWake;
    impl Wake for NoopWake {
        fn wake(self: Arc<Self>) {}
    }
    let waker = Waker::from(Arc::new(NoopWake));
    let mut cx = Context::from_waker(&waker);
    let mut fut = Box::pin(fut);
    loop {
        match fut.as_mut().poll(&mut cx) {
            Poll::Ready(v) => return v,
            Poll::Pending => std::thread::yield_now(),
        }
    }
}

#[cfg(all(feature = "async", feature = "graphics"))]
mod async_side {
    use super::{Kind, Transcript};
    use display_interface::DisplayError;
    use gc9a01::interface::AsyncWriteOnlyDataCommand;

    #[derive(Default)]
    pub struct AsyncFake {
        pub records: Transcript,
    }

    #[allow(clippy::unused_async, clippy::unused_async_trait_impl)]
    impl AsyncWriteOnlyDataCommand for AsyncFake {
        async fn send_commands(&mut self, cmd: &[u8]) -> Result<(), DisplayError> {
            self.records.push((Kind::Commands, cmd.to_vec()));
            Ok(())
        }
        async fn send_data(&mut self, data: &[u8]) -> Result<(), DisplayError> {
            self.records.push((Kind::Data, data.to_vec()));
            Ok(())
        }
        async fn send_pixel_data(&mut self, data: &[u16]) -> Result<(), DisplayError> {
            let bytes: Vec<u8> = data.iter().flat_map(|v| v.to_be_bytes()).collect();
            self.records.push((Kind::Data, bytes));
            Ok(())
        }
    }

    #[derive(Default)]
    pub struct AsyncDelay {
        pub total_ns: u64,
    }

    #[allow(clippy::unused_async, clippy::unused_async_trait_impl)]
    impl embedded_hal_async::delay::DelayNs for AsyncDelay {
        async fn delay_ns(&mut self, ns: u32) {
            self.total_ns += u64::from(ns);
        }
        async fn delay_us(&mut self, us: u32) {
            self.total_ns += u64::from(us) * 1_000;
        }
        async fn delay_ms(&mut self, ms: u32) {
            self.total_ns += u64::from(ms) * 1_000_000;
        }
    }
}

#[cfg(all(feature = "async", feature = "graphics"))]
#[test]
fn blocking_async_parity() {
    use async_side::{AsyncDelay, AsyncFake};
    use embedded_graphics_core::draw_target::DrawTarget;
    use embedded_graphics_core::geometry::{Point, Size};
    use embedded_graphics_core::pixelcolor::Rgb565;
    use embedded_graphics_core::prelude::RgbColor;
    use embedded_graphics_core::primitives::Rectangle;
    use gc9a01::Gc9a01Async;

    // Blocking run
    let (blocking_transcript, blocking_delay_ns) = {
        let mut d = Gc9a01::new(
            BlockingFake::default(),
            DisplayResolution240x240,
            DisplayRotation::Rotate0,
        )
        .into_buffered_graphics();
        let mut delay = BlockingDelay::default();
        d.init(&mut delay).unwrap();
        d.set_display_rotation(DisplayRotation::Rotate90).unwrap();
        d.set_brightness(Brightness::BRIGHT).unwrap();
        d.fill_solid(
            &Rectangle::new(Point::new(200, 200), Size::new(100, 100)),
            Rgb565::RED,
        )
        .unwrap();
        d.flush().unwrap();
        d.clear();
        d.flush().unwrap();
        let delay_ns = delay.total_ns;
        (d.release().records, delay_ns)
    };

    // Async run, same script
    let (async_transcript, async_delay_ns) = {
        let mut d = Gc9a01Async::new(
            AsyncFake::default(),
            DisplayResolution240x240,
            DisplayRotation::Rotate0,
        )
        .into_buffered_graphics();
        let mut delay = AsyncDelay::default();
        block_on(d.init(&mut delay)).unwrap();
        block_on(d.set_display_rotation(DisplayRotation::Rotate90)).unwrap();
        block_on(d.set_brightness(Brightness::BRIGHT)).unwrap();
        d.fill_solid(
            &Rectangle::new(Point::new(200, 200), Size::new(100, 100)),
            Rgb565::RED,
        )
        .unwrap();
        block_on(d.flush()).unwrap();
        d.clear();
        block_on(d.flush()).unwrap();
        let delay_ns = delay.total_ns;
        (d.release().records, delay_ns)
    };

    assert_eq!(blocking_transcript, async_transcript);
    assert_eq!(blocking_delay_ns, async_delay_ns);
    assert_eq!(blocking_delay_ns, 240_000_000);
}

#[cfg(not(all(feature = "async", feature = "graphics")))]
#[test]
fn parity_requires_async_and_graphics() {
    // Transcript parity is exercised under `--features async` (graphics on).
    let mut d = Gc9a01::new(
        BlockingFake::default(),
        DisplayResolution240x240,
        DisplayRotation::Rotate0,
    );
    let mut delay = BlockingDelay::default();
    d.init(&mut delay).unwrap();
    assert_eq!(delay.total_ns, 240_000_000);
}
