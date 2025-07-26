use crate::{NoStdIoError, OffsetWrite, Read, Result, Seek, SeekFrom, Write, offset::OffsetRead};

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
            .ok_or(NoStdIoError::UnexpectedEof)?;
        let buf_slice = buf
            .get_mut(0..data_slice.len())
            .ok_or(NoStdIoError::Other)?;
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
        let end = (start + buf.len()).min(self.data.len());
        let data_slice = self.data.get_mut(start..end).ok_or(NoStdIoError::Other)?;
        let buf_slice = buf.get(0..data_slice.len()).ok_or(NoStdIoError::Other)?;
        data_slice.copy_from_slice(buf_slice);
        self.pos = end;
        Ok(buf_slice.len())
    }
}

impl<'a> Seek for Cursor<'a> {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        match pos {
            SeekFrom::Start(v) => {
                self.pos = v.try_into().map_err(|_| NoStdIoError::InvalidInput)?;
                Ok(v)
            }
            SeekFrom::End(v) => {
                let len = self.data.len();
                if let Some(new_pos) =
                    len.checked_add_signed(v.try_into().map_err(|_| NoStdIoError::InvalidInput)?)
                {
                    self.pos = new_pos;
                    Ok(new_pos as u64)
                } else {
                    Err(NoStdIoError::InvalidInput)
                }
            }
            SeekFrom::Current(v) => {
                if let Some(new_pos) = self
                    .pos
                    .checked_add_signed(v.try_into().map_err(|_| NoStdIoError::InvalidInput)?)
                {
                    self.pos = new_pos;
                    Ok(new_pos as u64)
                } else {
                    Err(NoStdIoError::InvalidInput)
                }
            }
        }
    }
}

impl<'a> OffsetRead for Cursor<'a> {
    fn read_at_offset(&self, offset: u64, buf: &mut [u8]) -> Result<usize> {
        let start: usize = offset.try_into().map_err(|_| NoStdIoError::InvalidInput)?;
        if start > self.data.len() {
            return Ok(0);
        }
        let end = (start + buf.len()).min(self.data.len());
        let data_slice = self
            .data
            .get(start..end)
            .ok_or(NoStdIoError::UnexpectedEof)?;
        let buf_slice = buf
            .get_mut(0..data_slice.len())
            .ok_or(NoStdIoError::Other)?;
        buf_slice.copy_from_slice(data_slice);
        Ok(data_slice.len())
    }
}

impl<'a> OffsetWrite for Cursor<'a> {
    fn write_at_offset(&mut self, offset: u64, buf: &[u8]) -> Result<usize> {
        let start: usize = offset.try_into().map_err(|_| NoStdIoError::InvalidInput)?;
        if start > self.data.len() {
            return Ok(0);
        }
        let end = (start + buf.len()).min(self.data.len());
        let data_slice = self.data.get_mut(start..end).ok_or(NoStdIoError::Other)?;
        let buf_slice = buf.get(0..data_slice.len()).ok_or(NoStdIoError::Other)?;
        data_slice.copy_from_slice(buf_slice);
        Ok(buf_slice.len())
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
        assert_eq!(1, cursor.write(&buf).unwrap());
    }

    #[test]
    fn test_read() {
        let mut data = [0; 100];
        for i in 0..data.len() {
            data[i] = i as u8;
        }

        let mut cursor = Cursor::new(&mut data);

        let mut buf = [0; 10];
        assert_eq!(10, cursor.read(&mut buf).unwrap());
        for i in 0..10 {
            assert_eq!(i as u8, buf[i]);
        }

        let mut buf = [0; 100];
        assert_eq!(90, cursor.read(&mut buf).unwrap());
        for i in 10..90 {
            assert_eq!(i as u8, buf[i - 10]);
        }
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

    #[test]
    fn test_offset_read() {
        let mut data = [0; 100];
        for i in 0..data.len() {
            data[i] = i as u8;
        }

        let cursor = Cursor::new(&mut data);

        let mut buf = [0; 10];
        assert_eq!(10, cursor.read_at_offset(0, &mut buf).unwrap());
        for i in 0..10 {
            assert_eq!(i as u8, buf[i]);
        }

        let mut buf = [0; 100];
        assert_eq!(90, cursor.read_at_offset(10, &mut buf).unwrap());
        for i in 10..90 {
            assert_eq!(i as u8, buf[i - 10]);
        }

        assert_eq!(0, cursor.read_at_offset(100, &mut buf).unwrap());
    }

    #[test]
    fn test_offset_write() {
        let mut data = [0; 100];
        {
            let mut cursor = Cursor::new(&mut data);

            let mut buf = [0; 10];
            for i in 0..buf.len() {
                buf[i] = i as u8;
            }
            assert_eq!(10, cursor.write_at_offset(0, &mut buf).unwrap());

            let mut buf = [0; 100];
            for i in 0..buf.len() {
                buf[i] = i as u8;
            }
            assert_eq!(90, cursor.write_at_offset(10, &mut buf).unwrap());

            assert_eq!(0, cursor.write_at_offset(100, &mut buf).unwrap());
        }

        for i in 0..10 {
            assert_eq!(i as u8, data[i]);
        }
        for i in 10..90 {
            assert_eq!((i - 10) as u8, data[i]);
        }
    }
}
