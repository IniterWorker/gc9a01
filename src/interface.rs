//! Async transport trait (feature `async`).

#[cfg(feature = "async")]
use display_interface::DisplayError;

/// Async write-only data/command transport.
///
/// Mirrors blocking `display_interface::WriteOnlyDataCommand` but with
/// plain slice arguments; error type is shared so blocking and async
/// driver signatures differ only in `async`/`.await`.
#[cfg(feature = "async")]
#[allow(async_fn_in_trait)]
pub trait AsyncWriteOnlyDataCommand {
    /// Send command bytes (D/C low semantics owned by implementation).
    ///
    /// # Errors
    /// Returns `DisplayError` on transport failure.
    async fn send_commands(&mut self, cmd: &[u8]) -> Result<(), DisplayError>;

    /// Send data bytes (D/C high semantics owned by implementation).
    ///
    /// # Errors
    /// Returns `DisplayError` on transport failure.
    async fn send_data(&mut self, data: &[u8]) -> Result<(), DisplayError>;

    /// Send pixel data as big-endian `u16` stream.
    ///
    /// Default implementation chunks through a fixed 64-byte stack buffer
    /// converting with `to_be_bytes`; implementations may override.
    ///
    /// # Errors
    /// Returns `DisplayError` on transport failure.
    async fn send_pixel_data(&mut self, data: &[u16]) -> Result<(), DisplayError> {
        let mut buf = [0u8; 64];
        for chunk in data.chunks(32) {
            for (i, v) in chunk.iter().enumerate() {
                let bytes = v.to_be_bytes();
                buf[i * 2] = bytes[0];
                buf[i * 2 + 1] = bytes[1];
            }
            self.send_data(&buf[..chunk.len() * 2]).await?;
        }
        Ok(())
    }
}
