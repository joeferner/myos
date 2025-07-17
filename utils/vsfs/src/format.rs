use zerocopy::IntoBytes;

use crate::{
    BLOCK_SIZE, Error, INODES_PER_BLOCK, MAGIC, Result, SuperBlock,
    io::{ReadWriteSeek, SeekFrom},
    utils::div_ceil,
};

pub struct FormatVolumeOptions {
    pub inode_count: u32,
    pub data_block_count: u32,
}

impl FormatVolumeOptions {
    pub fn new(inode_count: u32, data_block_count: u32) -> Self {
        Self {
            inode_count,
            data_block_count,
        }
    }
}

pub fn format_volume<T: ReadWriteSeek>(file: &mut T, options: FormatVolumeOptions) -> Result<u64> {
    file.seek(SeekFrom::Start(0))?;

    let mut block = [0; BLOCK_SIZE];

    // write super block
    let super_block = SuperBlock {
        magic: MAGIC,
        inode_count: options.inode_count,
        data_block_count: options.data_block_count,
    };
    super_block
        .write_to_prefix(&mut block)
        .map_err(|_| Error::SizeError)?;
    file.write(&block)?;

    // write inode bitmap
    block.fill(0);
    let inode_bitmap_block_count = div_ceil(div_ceil(options.inode_count, 8), BLOCK_SIZE as u32);
    for _ in 0..inode_bitmap_block_count {
        file.write(&block)?;
    }

    // write data bitmap
    let data_bitmap_block_count =
        div_ceil(div_ceil(options.data_block_count, 8), BLOCK_SIZE as u32);
    for _ in 0..data_bitmap_block_count {
        file.write(&block)?;
    }

    // write inodes
    let inode_block_count = div_ceil(options.inode_count, INODES_PER_BLOCK as u32);
    for _ in 0..inode_block_count {
        file.write(&block)?;
    }

    // write data blocks
    for _ in 0..options.data_block_count {
        file.write(&block)?;
    }

    let size = file.seek(SeekFrom::End(0))?;
    file.seek(SeekFrom::Start(0))?;
    Ok(size)
}

#[cfg(test)]
mod tests {
    use crate::io::Cursor;

    use super::*;

    #[test]
    fn test_minimums() {
        let mut data = [0; 100 * BLOCK_SIZE];
        let mut cursor = Cursor::new(&mut data);
        let options = FormatVolumeOptions::new(1, 1);
        let size = format_volume(&mut cursor, options).unwrap();
        assert_eq!(10, size);
    }
}
