use crate::Result;

pub trait OffsetRead {
    /// Read data from self into the given buffer starting from the given offset.
    /// Returns the number of bytes read from source into buffer.
    fn read_at_offset(&self, offset: u64, buf: &mut [u8]) -> Result<usize>;
}

pub trait OffsetWrite {
    /// Writes data from the given buffer into self starting at the given offset.
    /// Returns the number of bytes written to self.
    fn write_at_offset(&mut self, offset: u64, buf: &[u8]) -> Result<usize>;
}
