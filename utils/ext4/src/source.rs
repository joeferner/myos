use myos_api::filesystem::{FilePos, Result};

pub trait Ext4Source {
    fn read(&self, file_pos: FilePos, buf: &mut [u8]) -> Result<()>;
}

#[cfg(any(test, feature = "std"))]
pub struct FileExt4Source {
    file: spin::Mutex<std::fs::File>,
}

#[cfg(any(test, feature = "std"))]
impl FileExt4Source {
    pub fn new(file: std::fs::File) -> Self {
        Self {
            file: spin::Mutex::new(file),
        }
    }
}

#[cfg(any(test, feature = "std"))]
impl Ext4Source for FileExt4Source {
    fn read(&self, file_pos: FilePos, buf: &mut [u8]) -> Result<()> {
        use std::io::{Read, Seek, SeekFrom};

        use myos_api::filesystem::FileIoError;
        use nostdio::NoStdIoError;

        let mut file = self.file.lock();
        file.seek(SeekFrom::Start(file_pos.0))
            .map_err(|err| FileIoError::IoError(NoStdIoError::StdIoError(err)))?;
        let read = file
            .read(buf)
            .map_err(|err| FileIoError::IoError(NoStdIoError::StdIoError(err)))?;
        if read != buf.len() {
            return Err(FileIoError::IoError(NoStdIoError::create_partial_read_error(
                file_pos.0,
                read,
                buf.len(),
            )));
        }
        Ok(())
    }
}
