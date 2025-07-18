use zerocopy::IntoBytes;

use crate::{
    BLOCK_SIZE, Error, INODE_SIZE, INode, Layout, MAGIC, MODE_DIRECTORY, ROOT_INODE_ID, ROOT_UID,
    Result, SuperBlock,
    directory::PhysicalDirectoryEntry,
    io::{ReadWriteSeek, SeekFrom},
};

pub struct FormatVolumeOptions {
    pub inode_count: u32,
    pub data_block_count: u32,
    pub time: u32,
}

impl FormatVolumeOptions {
    pub fn new(inode_count: u32, data_block_count: u32) -> Self {
        Self {
            inode_count,
            data_block_count,
            time: 0,
        }
    }
}

pub fn format_volume<T: ReadWriteSeek>(file: &mut T, options: FormatVolumeOptions) -> Result<u64> {
    file.seek(SeekFrom::Start(0))?;

    let mut block = [0; BLOCK_SIZE];
    let mut block_count: u64 = 0;

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
    block_count += 1;

    let layout = Layout::new(options.inode_count, options.data_block_count);

    // write inode bitmap
    block.fill(0);
    for i in 0..layout.inode_bitmap_block_count {
        if i == 0 {
            block[0] = 0x01;
        }
        file.write(&block)?;
        if i == 0 {
            block[0] = 0;
        }
        block_count += 1;
    }

    // write data bitmap
    for _ in 0..layout.data_bitmap_block_count {
        file.write(&block)?;
        block_count += 1;
    }

    // write inodes
    for i in 0..layout.inode_block_count {
        if i == 0 {
            let offset: usize = ROOT_INODE_ID as usize * INODE_SIZE;
            let mut root_inode = INode::new(0o755 | MODE_DIRECTORY, options.time);
            root_inode.uid = ROOT_UID;
            root_inode.gid = ROOT_UID;
            root_inode.blocks[0] = 0;
            root_inode
                .write_to_prefix(&mut block[offset..])
                .map_err(|_| Error::SizeError)?;
        }

        file.write(&block)?;

        if i == 0 {
            block.fill(0);
        }

        block_count += 1;
    }

    // write data blocks
    for i in 0..options.data_block_count {
        if i == 0 {
            let mut offset = 0;
            offset += PhysicalDirectoryEntry::write(&mut block[offset..], ROOT_INODE_ID, ".")?;
            PhysicalDirectoryEntry::write(&mut block[offset..], ROOT_INODE_ID, "..")?;
        }

        file.write(&block)?;

        if i == 0 {
            block.fill(0);
        }

        block_count += 1;
    }

    file.seek(SeekFrom::End(0))?;

    Ok(block_count * BLOCK_SIZE as u64)
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
        assert_eq!(20480, size);
    }
}
