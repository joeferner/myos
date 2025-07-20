use crate::{IoError, Read, Seek, SeekFrom, Write, error::Result};

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
        if start > self.data.len() {
            return Ok(0);
        }
        let end = (self.pos + buf.len()).min(self.data.len());
        let data_slice = self
            .data
            .get(start..end)
            .ok_or(IoError::Other("slice out of range"))?;
        let buf_slice = buf
            .get_mut(0..data_slice.len())
            .ok_or(IoError::Other("slice out of range"))?;
        buf_slice.copy_from_slice(data_slice);
        self.pos += data_slice.len();
        Ok(data_slice.len())
    }
}

impl<'a> Write for Cursor<'a> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let start = self.pos;
        if start > self.data.len() {
            return Ok(0);
        }
        let end = self.pos + buf.len();
        if end > self.data.len() {
            return Err(IoError::Other("write past end of array"));
        }
        let data_slice = self
            .data
            .get_mut(start..end)
            .ok_or(IoError::Other("slice out of range"))?;
        data_slice.copy_from_slice(buf);
        self.pos = end;
        Ok(buf.len())
    }
}

impl<'a> Seek for Cursor<'a> {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        match pos {
            SeekFrom::Start(v) => {
                self.pos = v.try_into()?;
                Ok(v)
            }
            SeekFrom::End(v) => {
                let len = self.data.len();
                if let Some(new_pos) = len.checked_add_signed(v.try_into()?) {
                    self.pos = new_pos;
                    Ok(new_pos as u64)
                } else {
                    Err(IoError::Other("seek end out of range"))
                }
            }
            SeekFrom::Current(v) => {
                if let Some(new_pos) = self.pos.checked_add_signed(v.try_into()?) {
                    self.pos = new_pos;
                    Ok(new_pos as u64)
                } else {
                    Err(IoError::Other("seek current out of range"))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seeks() {
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

        cursor.seek(SeekFrom::Start(101)).unwrap();
        assert_eq!(0, cursor.read(&mut buf).unwrap());
    }
}
