/// The `WouldBlockError` error indicates that the serial device was not ready immediately.
#[non_exhaustive]
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct WouldBlockError;

impl core::fmt::Display for WouldBlockError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("serial device not ready")
    }
}
