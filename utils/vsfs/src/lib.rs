#![cfg_attr(all(not(feature = "std"), not(test)), no_std)]
#![allow(clippy::new_without_default)]
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unimplemented,
    clippy::unreachable,
    clippy::indexing_slicing,
    clippy::cast_possible_truncation
)]

mod format;
mod inode;
mod layout;
mod physical;

use file_io::{FileIoError, Mode, Result, TimeSeconds};
pub use format::{FormatVolumeOptions, format_volume};
use io::ReadWriteSeek;

use crate::{
    inode::INode,
    layout::Layout,
    physical::{PhysicalSuperBlock, BLOCK_SIZE, MAGIC},
};

pub(crate) struct DataBlockIndex(pub u32);
pub(crate) struct INodeBlockIndex(pub u32);
pub(crate) const ROOT_INODE_IDX: INodeBlockIndex = INodeBlockIndex(2);

pub struct FsOptions {
    pub(crate) read_root_inode: bool,
}

impl FsOptions {
    pub fn new() -> Self {
        Self {
            read_root_inode: true,
        }
    }
}

pub struct FileSystem<T: ReadWriteSeek> {
    file: T,
    layout: Layout,
    root_inode: INode,
    block: [u8; BLOCK_SIZE],
}

impl<T: ReadWriteSeek> FileSystem<T> {
    pub fn new(mut file: T, options: FsOptions) -> Result<Self> {
        let mut block = [0; BLOCK_SIZE];
        file.seek(io::SeekFrom::Start(0))?;
        file.read(&mut block)?;
        let (super_block, _) =
            PhysicalSuperBlock::read_from_prefix(&block).map_err(|_| FileIoError::FormatError)?;
        if super_block.magic != MAGIC {
            return Err(FileIoError::FormatError);
        }

        let layout = Layout::new(super_block.inode_count, super_block.data_block_count);

        let mut fs = Self {
            file,
            layout,
            root_inode: INode::new(Mode(0o755), TimeSeconds(0)),
            block: [0; BLOCK_SIZE],
        };

        if options.read_root_inode {
            fs.root_inode = fs.read_inode(ROOT_INODE_IDX)?
        };

        Ok(fs)
    }
}

#[cfg(test)]
mod tests {
    use file_io::TimeSeconds;
    use io::Cursor;
    use myos_api::ROOT_UID;

    use crate::physical::BLOCK_SIZE;

    use super::*;

    #[test]
    fn test_root_dir() {
        let mut data = [0; 100 * BLOCK_SIZE];
        let cursor = Cursor::new(&mut data);
        let mut options = FormatVolumeOptions::new(10, 10);
        options.time = TimeSeconds(123);
        let mut fs = format_volume(cursor, options).unwrap();

        let root = fs.root_dir();
        assert_eq!(ROOT_UID, root.uid());
        assert_eq!(ROOT_UID, root.gid());
        assert_eq!(0o755, root.mode());

        let mut count = 0;
        for entry in root.iter(&mut fs).unwrap() {
            let entry = entry.unwrap();
            assert!(entry.is_dir());
            let dir = entry.to_dir().unwrap();
            assert_eq!(ROOT_UID, dir.uid());
            assert_eq!(ROOT_UID, dir.gid());
            assert_eq!(0o755, dir.mode());
            assert_eq!(ROOT_INODE_IDX, dir.inode_idx());

            if count == 0 {
                assert_eq!(".", entry.file_name().unwrap());
            } else if count == 1 {
                assert_eq!("..", entry.file_name().unwrap());
            }
            count += 1;
        }
        assert_eq!(2, count);
    }

    #[test]
    fn test_create_file() {
        let mut data = [0; 100 * BLOCK_SIZE];
        let cursor = Cursor::new(&mut data);
        let mut fs = format_volume(cursor, FormatVolumeOptions::new(10, 10)).unwrap();

        let mut root_dir = fs.root_dir();
        let mut file = root_dir
            .create_file(
                &mut fs,
                CreateFileOptions {
                    file_name: "hello.txt",
                    uid: ROOT_UID,
                    gid: ROOT_UID,
                    mode: 0o755,
                    time: 123,
                },
            )
            .unwrap();
        file.write_all(b"Hello World!").unwrap();
    }

    // TODO test inode exhaustion
}
