#![cfg_attr(all(not(feature = "std"), not(test)), no_std)]
#![allow(clippy::new_without_default)]
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unimplemented,
    clippy::unreachable,
    clippy::indexing_slicing,
    clippy::cast_possible_truncation
)]

mod cursor;
mod error;

pub use cursor::Cursor;
pub use error::{IoError, Result};

/// Enumeration of possible methods to seek within an I/O object.
///
/// It is based on the `std::io::SeekFrom` enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeekFrom {
    /// Sets the offset to the provided number of bytes.
    Start(u64),

    /// Sets the offset to the size of this object plus the specified number of
    /// bytes.
    ///
    /// It is possible to seek beyond the end of an object, but it's an error to
    /// seek before byte 0.
    End(i64),

    /// Sets the offset to the current position plus the specified number of
    /// bytes.
    ///
    /// It is possible to seek beyond the end of an object, but it's an error to
    /// seek before byte 0.
    Current(i64),
}

#[cfg(feature = "std")]
impl From<SeekFrom> for std::io::SeekFrom {
    fn from(value: SeekFrom) -> Self {
        match value {
            SeekFrom::Start(v) => std::io::SeekFrom::Start(v),
            SeekFrom::End(v) => std::io::SeekFrom::End(v),
            SeekFrom::Current(v) => std::io::SeekFrom::Current(v),
        }
    }
}

pub trait Read {
    /// Pull some bytes from this source into the specified buffer, returning
    /// how many bytes were read.
    ///
    /// This function does not provide any guarantees about whether it blocks
    /// waiting for data, but if an object needs to block for a read and cannot,
    /// it will typically signal this via an [`Err`] return value.
    ///
    /// If the return value of this method is [`Ok(n)`], then implementations must
    /// guarantee that `0 <= n <= buf.len()`. A nonzero `n` value indicates
    /// that the buffer `buf` has been filled in with `n` bytes of data from this
    /// source. If `n` is `0`, then it can indicate one of two scenarios:
    ///
    /// 1. This reader has reached its "end of file" and will likely no longer
    ///    be able to produce bytes. Note that this does not mean that the
    ///    reader will *always* no longer be able to produce bytes. As an example,
    ///    on Linux, this method will call the `recv` syscall for a [`TcpStream`],
    ///    where returning zero indicates the connection was shut down correctly. While
    ///    for [`File`], it is possible to reach the end of file and get zero as result,
    ///    but if more data is appended to the file, future calls to `read` will return
    ///    more data.
    /// 2. The buffer specified was 0 bytes in length.
    ///
    /// It is not an error if the returned value `n` is smaller than the buffer size,
    /// even when the reader is not at the end of the stream yet.
    /// This may happen for example because fewer bytes are actually available right now
    /// (e. g. being close to end-of-file) or because read() was interrupted by a signal.
    ///
    /// As this trait is safe to implement, callers in unsafe code cannot rely on
    /// `n <= buf.len()` for safety.
    /// Extra care needs to be taken when `unsafe` functions are used to access the read bytes.
    /// Callers have to ensure that no unchecked out-of-bounds accesses are possible even if
    /// `n > buf.len()`.
    ///
    /// *Implementations* of this method can make no assumptions about the contents of `buf` when
    /// this function is called. It is recommended that implementations only write data to `buf`
    /// instead of reading its contents.
    ///
    /// Correspondingly, however, *callers* of this method in unsafe code must not assume
    /// any guarantees about how the implementation uses `buf`. The trait is safe to implement,
    /// so it is possible that the code that's supposed to write to the buffer might also read
    /// from it. It is your responsibility to make sure that `buf` is initialized
    /// before calling `read`. Calling `read` with an uninitialized `buf` (of the kind one
    /// obtains via [`MaybeUninit<T>`]) is not safe, and can lead to undefined behavior.
    ///
    /// [`MaybeUninit<T>`]: crate::mem::MaybeUninit
    ///
    /// # Errors
    ///
    /// If this function encounters any form of I/O or other error, an error
    /// variant will be returned. If an error is returned then it must be
    /// guaranteed that no bytes were read.
    ///
    /// An error of the [`ErrorKind::Interrupted`] kind is non-fatal and the read
    /// operation should be retried if there is nothing else to do.
    fn read(&mut self, buf: &mut [u8]) -> Result<usize>;
}

pub trait Write {
    /// Writes a buffer into this writer, returning how many bytes were written.
    ///
    /// This function will attempt to write the entire contents of `buf`, but
    /// the entire write might not succeed, or the write may also generate an
    /// error. Typically, a call to `write` represents one attempt to write to
    /// any wrapped object.
    ///
    /// Calls to `write` are not guaranteed to block waiting for data to be
    /// written, and a write which would otherwise block can be indicated through
    /// an [`Err`] variant.
    ///
    /// If this method consumed `n > 0` bytes of `buf` it must return [`Ok(n)`].
    /// If the return value is `Ok(n)` then `n` must satisfy `n <= buf.len()`.
    /// A return value of `Ok(0)` typically means that the underlying object is
    /// no longer able to accept bytes and will likely not be able to in the
    /// future as well, or that the buffer provided is empty.
    ///
    /// # Errors
    ///
    /// Each call to `write` may generate an I/O error indicating that the
    /// operation could not be completed. If an error is returned then no bytes
    /// in the buffer were written to this writer.
    ///
    /// It is **not** considered an error if the entire buffer could not be
    /// written to this writer.
    ///
    /// An error of the [`ErrorKind::Interrupted`] kind is non-fatal and the
    /// write operation should be retried if there is nothing else to do.
    fn write(&mut self, buf: &[u8]) -> Result<usize>;
}

pub trait Seek {
    /// Seek to an offset, in bytes, in a stream.
    ///
    /// A seek beyond the end of a stream is allowed, but behavior is defined
    /// by the implementation.
    ///
    /// If the seek operation completed successfully,
    /// this method returns the new position from the start of the stream.
    /// That position can be used later with [`SeekFrom::Start`].
    ///
    /// # Errors
    ///
    /// Seeking can fail, for example because it might involve flushing a buffer.
    ///
    /// Seeking to a negative offset is considered an error.
    fn seek(&mut self, pos: SeekFrom) -> Result<u64>;
}

#[cfg(feature = "std")]
impl Read for std::fs::File {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        (self as &mut dyn std::io::Read)
            .read(buf)
            .map_err(IoError::StdIoError)
    }
}

#[cfg(feature = "std")]
impl Write for std::fs::File {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        (self as &mut dyn std::io::Write)
            .write(buf)
            .map_err(IoError::StdIoError)
    }
}

#[cfg(feature = "std")]
impl Seek for std::fs::File {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        let new_offset = (self as &mut dyn std::io::Seek)
            .seek(pos.into())
            .map_err(IoError::StdIoError)?;
        Ok(new_offset)
    }
}

/// A sum of `Read`, `Write` and `Seek` traits.
pub trait ReadWriteSeek: Read + Write + Seek {}
impl<T: Read + Write + Seek> ReadWriteSeek for T {}
