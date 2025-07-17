use crate::{Error, Result};

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
impl Into<std::io::SeekFrom> for SeekFrom {
    fn into(self) -> std::io::SeekFrom {
        match self {
            SeekFrom::Start(v) => std::io::SeekFrom::Start(v),
            SeekFrom::End(v) => std::io::SeekFrom::End(v),
            SeekFrom::Current(v) => std::io::SeekFrom::Current(v),
        }
    }
}

pub trait Read {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize>;
}

pub trait Write {
    fn write(&mut self, buf: &[u8]) -> Result<usize>;
}

pub trait Seek {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64>;
}

#[cfg(feature = "std")]
impl Read for std::fs::File {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        (self as &mut dyn std::io::Read)
            .read(buf)
            .map_err(|e| crate::Error::StdIoError(e))
    }
}

#[cfg(feature = "std")]
impl Write for std::fs::File {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        (self as &mut dyn std::io::Write)
            .write(buf)
            .map_err(|e| crate::Error::StdIoError(e))
    }
}

#[cfg(feature = "std")]
impl Seek for std::fs::File {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        (self as &mut dyn std::io::Seek)
            .seek(pos.into())
            .map_err(|e| crate::Error::StdIoError(e))
    }
}

/// A sum of `Read`, `Write` and `Seek` traits.
pub trait ReadWriteSeek: Read + Write + Seek {}
impl<T: Read + Write + Seek> ReadWriteSeek for T {}

pub struct Cursor<'a> {
    data: &'a mut [u8],
    pos: usize,
}

impl<'a> Cursor<'a> {
    pub fn new(data: &'a mut [u8]) -> Self {
        Self { data, pos: 0 }
    }
}

impl<'a> Read for Cursor<'a> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let start = self.pos;
        let end = (self.pos + buf.len()).min(self.data.len());
        let data_slice = &self.data[start..end];
        let buf_slice = &mut buf[0..data_slice.len()];
        buf_slice.copy_from_slice(data_slice);
        self.pos += data_slice.len();
        Ok(data_slice.len())
    }
}

impl<'a> Write for Cursor<'a> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let start = self.pos;
        let end = self.pos + buf.len();
        if end > self.data.len() {
            return Err(Error::SizeError);
        }
        let data_slice = &mut self.data[start..end];
        data_slice.copy_from_slice(buf);
        self.pos = end;
        Ok(buf.len())
    }
}

impl<'a> Seek for Cursor<'a> {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        match pos {
            SeekFrom::Start(v) => {
                self.pos = v as usize;
                Ok(v)
            }
            SeekFrom::End(v) => {
                let len = self.data.len() as u64;
                if let Some(new_pos) = len.checked_add_signed(v) {
                    self.pos = new_pos as usize;
                    Ok(new_pos as u64)
                } else {
                    Err(Error::SizeError)
                }
            }
            SeekFrom::Current(v) => {
                let pos = self.pos as u64;
                if let Some(new_pos) = pos.checked_add_signed(v) {
                    self.pos = new_pos as usize;
                    Ok(new_pos as u64)
                } else {
                    Err(Error::SizeError)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor_seeks() {
        let mut data = [0; 100];
        let mut cursor = Cursor::new(&mut data);
        assert_eq!(0, cursor.seek(SeekFrom::Start(0)).unwrap());
        assert_eq!(100, cursor.seek(SeekFrom::End(0)).unwrap());
        assert_eq!(100, cursor.seek(SeekFrom::Current(0)).unwrap());
        assert_eq!(0, cursor.seek(SeekFrom::End(-100)).unwrap());
        assert!(cursor.seek(SeekFrom::End(-101)).is_err());

        assert_eq!(0, cursor.seek(SeekFrom::Start(0)).unwrap());
        assert!(cursor.seek(SeekFrom::Current(-1)).is_err());
    }

    #[test]
    fn test_write() {
        let mut data = [0; 100];
        let mut cursor = Cursor::new(&mut data);

        let buf = [1; 10];
        cursor.write(&buf).unwrap();
        assert_eq!(10, cursor.seek(SeekFrom::Current(0)).unwrap());

        let buf = [2; 10];
        cursor.write(&buf).unwrap();
        assert_eq!(20, cursor.seek(SeekFrom::Current(0)).unwrap());

        let mut buf = [9; 101];
        cursor.seek(SeekFrom::Start(0)).unwrap();
        assert_eq!(100, cursor.read(&mut buf).unwrap());
        for i in 0..buf.len() {
            if i < 10 {
                assert_eq!(1, buf[i]);
            } else if i < 20 {
                assert_eq!(2, buf[i]);
            } else if i < 100 {
                assert_eq!(0, buf[i]);
            } else {
                assert_eq!(9, buf[i]);
            }
        }
    }

    #[test]
    fn test_write_past_end() {
        let mut data = [0; 100];
        let mut cursor = Cursor::new(&mut data);

        let buf = [1; 10];
        cursor.seek(SeekFrom::Start(99)).unwrap();
        assert!(cursor.write(&buf).is_err());
    }

    #[test]
    fn test_read_past_end() {
        let mut data = [0; 100];
        let mut cursor = Cursor::new(&mut data);
        cursor.seek(SeekFrom::End(0)).unwrap();

        let mut buf = [0; 10];
        assert_eq!(0, cursor.read(&mut buf).unwrap());
    }
}
