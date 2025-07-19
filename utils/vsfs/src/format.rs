use file_io::{FileIoError, FilePos, MODE_DIRECTORY, Mode, Result, TimeSeconds};
use io::{ReadWriteSeek, SeekFrom};
use myos_api::ROOT_UID;

use crate::{
    layout::Layout, physical::{PhysicalDirectoryEntry, PhysicalSuperBlock, BLOCK_SIZE, MAGIC}, DataBlockIndex, FileSystem, FsOptions, INode, ROOT_INODE_IDX
};

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
) -> Result<FileSystem<T>> {
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
    fs_options.read_root_inode = false;
    let mut fs = FileSystem::new(file, fs_options)?;

    // write root directory data
    block.fill(0);
    let mut offset: usize = 0;
    offset += PhysicalDirectoryEntry::write(ROOT_INODE_IDX, ".", &mut block[offset..])?;
    offset += PhysicalDirectoryEntry::write(ROOT_INODE_IDX, "..", &mut block[offset..])?;
    fs.write_data_block(0, block)?;
    let data_size = offset as u64;

    // write root directory inode
    let mut root_inode = INode::new(Mode(0o755) | MODE_DIRECTORY, options.time);
    root_inode.uid = ROOT_UID;
    root_inode.gid = ROOT_UID;
    root_inode.size = FilePos(data_size);
    root_inode.blocks[0] = Some(DataBlockIndex(0));
    fs.write_inode(ROOT_INODE_IDX, root_inode)?;

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
                * BLOCK_SIZE as Addr,
            fs.size()
        );
    }
}
