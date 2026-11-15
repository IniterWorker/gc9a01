//! Test fakes: recording transport, delay, pin.
#![allow(clippy::unwrap_used, clippy::panic, dead_code)]

use display_interface::{DataFormat, DisplayError, WriteOnlyDataCommand};
use embedded_hal::delay::DelayNs;
use embedded_hal::digital::{ErrorType, OutputPin};
#[cfg(feature = "async")]
use std::future::Future;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordKind {
    Commands,
    Data,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Record {
    pub kind: RecordKind,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Default)]
pub struct FakeInterface {
    pub records: Vec<Record>,
    /// When >0, next `send_data` calls fail that many times (before recording).
    pub fail_data_remaining: usize,
}

impl FakeInterface {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn opcodes(&self) -> Vec<u8> {
        self.records
            .iter()
            .filter(|r| r.kind == RecordKind::Commands)
            .flat_map(|r| r.bytes.iter().copied())
            .collect()
    }
    pub fn data_bytes_total(&self) -> usize {
        self.records
            .iter()
            .filter(|r| r.kind == RecordKind::Data)
            .map(|r| r.bytes.len())
            .sum()
    }
    pub fn clear_records(&mut self) {
        self.records.clear();
    }
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
        _ => panic!("unsupported DataFormat in FakeInterface"),
    }
}

impl WriteOnlyDataCommand for FakeInterface {
    fn send_commands(&mut self, cmd: DataFormat<'_>) -> Result<(), DisplayError> {
        let bytes = expand(cmd);
        self.records.push(Record {
            kind: RecordKind::Commands,
            bytes,
        });
        Ok(())
    }
    fn send_data(&mut self, buf: DataFormat<'_>) -> Result<(), DisplayError> {
        if self.fail_data_remaining > 0 {
            self.fail_data_remaining -= 1;
            // Still consume iterator formats to avoid unused warnings? Expand & drop.
            let _ = expand(buf);
            return Err(DisplayError::BusWriteError);
        }
        let bytes = expand(buf);
        self.records.push(Record {
            kind: RecordKind::Data,
            bytes,
        });
        Ok(())
    }
}

#[derive(Debug, Default)]
#[allow(clippy::struct_field_names)]
pub struct FakeDelay {
    pub total_ns: u64,
    pub total_us: u64,
    pub total_ms: u64,
}

impl FakeDelay {
    pub fn new() -> Self {
        Self::default()
    }
    /// Total delay in ms derived from ns accumulator.
    pub fn total_ms_from_ns(&self) -> u64 {
        self.total_ns / 1_000_000
    }
}

impl DelayNs for FakeDelay {
    fn delay_ns(&mut self, ns: u32) {
        self.total_ns += u64::from(ns);
    }
    fn delay_us(&mut self, us: u32) {
        self.total_us += u64::from(us);
        self.total_ns += u64::from(us) * 1_000;
    }
    fn delay_ms(&mut self, ms: u32) {
        self.total_ms += u64::from(ms);
        self.total_ns += u64::from(ms) * 1_000_000;
    }
}

#[derive(Debug, Default)]
pub struct FakePin {
    pub levels: Vec<bool>,
    pub state: bool,
}

impl ErrorType for FakePin {
    type Error = core::convert::Infallible;
}

impl OutputPin for FakePin {
    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.state = false;
        self.levels.push(false);
        Ok(())
    }
    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.state = true;
        self.levels.push(true);
        Ok(())
    }
}

#[cfg(feature = "async")]
#[allow(clippy::unused_async, clippy::unused_async_trait_impl)]
pub mod async_fakes {
    use super::{Record, RecordKind};
    use crate::interface::AsyncWriteOnlyDataCommand;
    use display_interface::DisplayError;

    #[derive(Debug, Default)]
    pub struct FakeAsyncInterface {
        pub records: Vec<Record>,
        pub fail_data_remaining: usize,
    }

    impl FakeAsyncInterface {
        pub fn new() -> Self {
            Self::default()
        }
        pub fn opcodes(&self) -> Vec<u8> {
            self.records
                .iter()
                .filter(|r| r.kind == RecordKind::Commands)
                .flat_map(|r| r.bytes.iter().copied())
                .collect()
        }
        pub fn data_bytes_total(&self) -> usize {
            self.records
                .iter()
                .filter(|r| r.kind == RecordKind::Data)
                .map(|r| r.bytes.len())
                .sum()
        }
        pub fn clear_records(&mut self) {
            self.records.clear();
        }
    }

    impl AsyncWriteOnlyDataCommand for FakeAsyncInterface {
        #[allow(clippy::unused_async_trait_impl)]
        async fn send_commands(&mut self, cmd: &[u8]) -> Result<(), DisplayError> {
            self.records.push(Record {
                kind: RecordKind::Commands,
                bytes: cmd.to_vec(),
            });
            Ok(())
        }
        #[allow(clippy::unused_async_trait_impl)]
        async fn send_data(&mut self, data: &[u8]) -> Result<(), DisplayError> {
            if self.fail_data_remaining > 0 {
                self.fail_data_remaining -= 1;
                return Err(DisplayError::BusWriteError);
            }
            self.records.push(Record {
                kind: RecordKind::Data,
                bytes: data.to_vec(),
            });
            Ok(())
        }
        #[allow(clippy::unused_async_trait_impl)]
        async fn send_pixel_data(&mut self, data: &[u16]) -> Result<(), DisplayError> {
            if self.fail_data_remaining > 0 {
                self.fail_data_remaining -= 1;
                return Err(DisplayError::BusWriteError);
            }
            let bytes: Vec<u8> = data.iter().flat_map(|v| v.to_be_bytes()).collect();
            self.records.push(Record {
                kind: RecordKind::Data,
                bytes,
            });
            Ok(())
        }
    }

    #[derive(Debug, Default)]
    #[allow(clippy::struct_field_names)]
    pub struct FakeAsyncDelay {
        pub total_ns: u64,
        pub total_us: u64,
        pub total_ms: u64,
    }

    impl FakeAsyncDelay {
        pub fn total_ms_from_ns(&self) -> u64 {
            self.total_ns / 1_000_000
        }
    }

    impl embedded_hal_async::delay::DelayNs for FakeAsyncDelay {
        async fn delay_ns(&mut self, ns: u32) {
            self.total_ns += u64::from(ns);
        }
        async fn delay_us(&mut self, us: u32) {
            self.total_us += u64::from(us);
            self.total_ns += u64::from(us) * 1_000;
        }
        async fn delay_ms(&mut self, ms: u32) {
            self.total_ms += u64::from(ms);
            self.total_ns += u64::from(ms) * 1_000_000;
        }
    }
}

/// Minimal `block_on` for tests: safe `Wake`-based no-op waker.
/// Futures under test complete without wakeups (fakes are immediate).
#[cfg(feature = "async")]
pub fn block_on<F: Future>(fut: F) -> F::Output {
    use core::task::{Context, Poll};
    use std::sync::Arc;
    use std::task::{Wake, Waker};
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
