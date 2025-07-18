use zerocopy::IntoBytes;

use crate::{
    directory::PhysicalDirectoryEntry, io::{ReadWriteSeek, SeekFrom}, Addr, BlockIndex, Error, FileSize, FileSystem, FsOptions, INode, Layout, Result, SuperBlock, Time, BLOCK_SIZE, MAGIC, MODE_DIRECTORY, ROOT_INODE_ID, ROOT_UID
};

pub struct FormatVolumeOptions {
    pub inode_count: u32,
    pub data_block_count: u32,
    pub time: Time,
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

pub fn format_volume<T: ReadWriteSeek>(file: &mut T, options: FormatVolumeOptions) -> Result<Addr> {
    file.seek(SeekFrom::Start(0))?;

    let mut block = [0; BLOCK_SIZE];
    let mut block_count: BlockIndex = 0;

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
    for _ in 0..layout.inode_bitmap_block_count {
        file.write(&block)?;
        block_count += 1;
    }

    // write data bitmap
    for _ in 0..layout.data_bitmap_block_count {
        file.write(&block)?;
        block_count += 1;
    }

    // write inodes
    for _ in 0..layout.inode_block_count {
        file.write(&block)?;
        block_count += 1;
    }

    // write data blocks
    for _ in 0..options.data_block_count {
        file.write(&block)?;
        block_count += 1;
    }

    let mut fs_options = FsOptions::new();
    fs_options.read_root_inode = false;
    let mut fs = FileSystem::new(file, fs_options)?;

    // write root directory data
    block.fill(0);
    let mut offset = 0;
    offset += PhysicalDirectoryEntry::write(&mut block[offset..], ROOT_INODE_ID, ".")?;
    offset += PhysicalDirectoryEntry::write(&mut block[offset..], ROOT_INODE_ID, "..")?;
    fs.write_data_block(0, block)?;
    let data_size = offset;

    // write root directory inode
    let mut root_inode = INode::new(0o755 | MODE_DIRECTORY, options.time);
    root_inode.uid = ROOT_UID;
    root_inode.gid = ROOT_UID;
    root_inode.size = data_size as FileSize;
    root_inode.blocks[0] = 0;
    fs.write_inode(ROOT_INODE_ID, root_inode)?;

    file.seek(SeekFrom::End(0))?;

    Ok(block_count as Addr * BLOCK_SIZE as Addr)
}

#[cfg(test)]
mod tests {
    use crate::io::Cursor;

    use super::*;

    #[test]
    fn test_minimums() {
        let mut data = [0; 100 * BLOCK_SIZE];
        let mut cursor = Cursor::new(&mut data);
        let options = FormatVolumeOptions::new(10, 10);
        let size = format_volume(&mut cursor, options).unwrap();
        assert_eq!(
            (1 /* super block */ + 1 /* inode bitmap */ + 1 /* data bitmap */ + 1 /* inode data */ + 10/* data */)
                * BLOCK_SIZE as Addr,
            size
        );
    }
}
