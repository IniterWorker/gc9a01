//! Compile-only async buffered example over stub transport.
//! Sequence: new -> into_buffered_graphics -> init -> draw -> flush.

use display_interface::DisplayError;
use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
};
use gc9a01::{
    display::DisplayResolution240x240, interface::AsyncWriteOnlyDataCommand,
    rotation::DisplayRotation, Gc9a01Async,
};
use std::future::Future;

struct StubInterface;

impl AsyncWriteOnlyDataCommand for StubInterface {
    async fn send_commands(&mut self, _cmd: &[u8]) -> Result<(), DisplayError> {
        Ok(())
    }
    async fn send_data(&mut self, _data: &[u8]) -> Result<(), DisplayError> {
        Ok(())
    }
}

struct StubDelay;

impl embedded_hal_async::delay::DelayNs for StubDelay {
    async fn delay_ns(&mut self, _ns: u32) {}
}

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

fn main() {
    block_on(async {
        let mut display = Gc9a01Async::new(
            StubInterface,
            DisplayResolution240x240,
            DisplayRotation::Rotate0,
        )
        .into_buffered_graphics();
        display.init(&mut StubDelay).await.unwrap();
        Rectangle::new(Point::new(20, 20), Size::new(80, 60))
            .into_styled(PrimitiveStyle::with_fill(Rgb565::RED))
            .draw(&mut display)
            .unwrap();
        display.flush().await.unwrap();
    });
}
