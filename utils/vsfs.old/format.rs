use file_io::{FileIoError, Result, TimeSeconds};
use io::{ReadWriteSeek, SeekFrom};

use crate::{
    Vsfs, FsOptions,
    layout::Layout,
    physical::{BLOCK_SIZE, MAGIC, PhysicalSuperBlock},
};
use zerocopy::IntoBytes;

pub struct FormatVolumeOptions {
    pub inode_count: u32,
    pub data_block_count: u32,
    pub time: TimeSeconds,
}

impl FormatVolumeOptions {
    pub fn new(inode_count: u32, data_block_count: u32) -> Self {
        Self {
            inode_count,
            data_block_count,
            time: TimeSeconds(0),
        }
    }
}

pub fn format_volume<T: ReadWriteSeek>(
    mut file: T,
    options: FormatVolumeOptions,
) -> Result<Vsfs<T>> {
    file.seek(SeekFrom::Start(0))?;

    let mut block = [0; BLOCK_SIZE];

    // write super block
    let super_block = PhysicalSuperBlock {
        magic: MAGIC,
        inode_count: options.inode_count,
        data_block_count: options.data_block_count,
    };
    super_block
        .write_to_prefix(&mut block)
        .map_err(|_| FileIoError::BufferTooSmall)?;
    file.write(&block)?;

    let layout = Layout::new(options.inode_count, options.data_block_count);

    // write inode bitmap
    block.fill(0);
    for _ in 0..layout.inode_bitmap_block_count {
        file.write(&block)?;
    }

    // write data bitmap
    for _ in 0..layout.data_bitmap_block_count {
        file.write(&block)?;
    }

    // write inodes
    for _ in 0..layout.inode_block_count {
        file.write(&block)?;
    }

    // write data blocks
    for _ in 0..options.data_block_count {
        file.write(&block)?;
    }

    let mut fs_options = FsOptions::new();
    fs_options.init_root_inode = true;
    fs_options.init_root_inode_time = options.time;
    let fs = Vsfs::new(file, fs_options)?;

    Ok(fs)
}

#[cfg(test)]
mod tests {
    use io::Cursor;

    use super::*;

    #[test]
    fn test_minimums() {
        let mut data = [0; 100 * BLOCK_SIZE];
        let cursor = Cursor::new(&mut data);
        let options = FormatVolumeOptions::new(10, 10);
        let fs = format_volume(cursor, options).unwrap();
        assert_eq!(
            (1 /* super block */ + 1 /* inode bitmap */ + 1 /* data bitmap */ + 1 /* inode data */ + 10/* data */)
                * BLOCK_SIZE as u64,
            fs.size().0
        );
    }
}
