use crate::{io::ReadWriteSeek, Result};

pub struct FormatVolumeOptions {}

impl FormatVolumeOptions {
    pub fn new() -> Self {
        Self {}
    }
}

pub fn format_volume<T: ReadWriteSeek>(file: &T, options: FormatVolumeOptions) -> Result<()> {
    todo!();
}

