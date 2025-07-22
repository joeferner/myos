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

mod directory;
mod file;
mod format;
mod inode;
mod layout;
mod physical;

pub use directory::CreateFileOptions;
use file_io::{FileIoError, FilePos, Mode, Result, TimeSeconds};
pub use format::{FormatVolumeOptions, format_volume};
use io::{IoError, ReadWriteSeek, SeekFrom};
use myos_api::Uid;
use zerocopy::{FromBytes, IntoBytes};

use crate::{
    directory::Directory,
    inode::INode,
    layout::Layout,
    physical::{
        BLOCK_NOT_SET, BLOCK_SIZE, IMMEDIATE_BLOCK_COUNT, MAGIC, PHYSICAL_INODE_SIZE,
        PHYSICAL_INODES_PER_BLOCK, PhysicalDirectoryEntry, PhysicalINode, PhysicalSuperBlock,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct DataBlockIndex(pub u32);

impl DataBlockIndex {
    pub(crate) fn from_u32(v: u32) -> Option<Self> {
        if v == BLOCK_NOT_SET {
            None
        } else {
            Some(Self(v))
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct INodeBlockIndex(pub u32);

impl INodeBlockIndex {
    pub(crate) fn root() -> Self {
        INodeBlockIndex(2)
    }
}

pub struct FsOptions {
    pub(crate) init_root_inode: bool,
    pub(crate) init_root_inode_time: TimeSeconds,
}

impl FsOptions {
    pub fn new() -> Self {
        Self {
            init_root_inode: false,
            init_root_inode_time: TimeSeconds(0),
        }
    }
}

pub struct Vsfs<T: ReadWriteSeek> {
    file: T,
    layout: Layout,
    block: [u8; BLOCK_SIZE],
}

impl<T: ReadWriteSeek> Vsfs<T> {
    pub fn new(mut file: T, options: FsOptions) -> Result<Self> {
        let mut block = [0; BLOCK_SIZE];
        file.seek(io::SeekFrom::Start(0))?;
        file.read(&mut block)?;
        let (super_block, _) = PhysicalSuperBlock::read_from_prefix(&block)
            .map_err(|_| FileIoError::BufferTooSmall)?;
        if super_block.magic != MAGIC {
            return Err(FileIoError::Other("invalid magic"));
        }

        let layout = Layout::new(super_block.inode_count, super_block.data_block_count);

        let mut fs = Self {
            file,
            layout,
            block: [0; BLOCK_SIZE],
        };

        if options.init_root_inode {
            // write root directory inode
            let mut root_inode = INode::new(
                Mode(0o755) | Mode::directory(),
                options.init_root_inode_time,
            );
            root_inode.uid = Uid::root();
            root_inode.gid = Uid::root();
            root_inode.size = FilePos(0);
            root_inode.blocks[0] = Some(DataBlockIndex(0));
            fs.write_inode(INodeBlockIndex::root(), root_inode)?;

            // write root directory data
            let mut buf = [0; BLOCK_SIZE];

            let dir_entry_buf =
                PhysicalDirectoryEntry::write(INodeBlockIndex::root(), ".", &mut buf)?;
            fs.write(INodeBlockIndex::root(), ReadWritePos::End(0), dir_entry_buf)?;

            let dir_entry_buf =
                PhysicalDirectoryEntry::write(INodeBlockIndex::root(), "..", &mut buf)?;
            fs.write(INodeBlockIndex::root(), ReadWritePos::End(0), dir_entry_buf)?;
        }

        Ok(fs)
    }

    pub fn size(&self) -> FilePos {
        self.layout.size()
    }

    /// Reads an inode.
    ///
    /// This function will validate that the given index has data.
    fn read_inode(&mut self, inode_idx: INodeBlockIndex) -> Result<INode> {
        if !self.is_inode_idx_readable(inode_idx)? {
            return Err(FileIoError::Other("cannot ready from empty inode"));
        }

        let (block_addr, inode_offset) = self.layout.calc_inode_block_addr(inode_idx)?;
        self.file.seek(SeekFrom::Start(block_addr.0))?;
        if self.file.read(&mut self.block)? != BLOCK_SIZE {
            return Err(FileIoError::BufferTooSmall);
        }
        let buf = self
            .block
            .get(inode_offset..inode_offset + PHYSICAL_INODE_SIZE)
            .ok_or(FileIoError::BufferTooSmall)?;
        let inode = PhysicalINode::read_from_bytes(buf).map_err(|_| FileIoError::BufferTooSmall)?;
        Ok(inode.into())
    }

    /// Checks the inode bitmap to see if the given inode has data
    fn is_inode_idx_readable(&mut self, inode_idx: INodeBlockIndex) -> Result<bool> {
        let (addr, offset, bit) = self.layout.calc_inode_bitmap_addr(inode_idx)?;
        self.file.seek(SeekFrom::Start(addr.0))?;
        self.file.read(&mut self.block)?;
        Ok((self.block[offset] >> bit) == 1)
    }

    /// Writes an inode at the given index.
    ///
    /// This function overwrites any existing inode data and updates the inode
    /// bitmap to indicate that the inode is now filled
    fn write_inode(&mut self, inode_idx: INodeBlockIndex, inode: INode) -> Result<()> {
        // write inode
        let (addr, offset) = self.layout.calc_inode_block_addr(inode_idx)?;
        self.file.seek(SeekFrom::Start(addr.0))?;
        self.file.read(&mut self.block)?;
        let buf = self
            .block
            .get_mut(offset..offset + PHYSICAL_INODE_SIZE)
            .ok_or(FileIoError::BufferTooSmall)?;
        let physical_inode: PhysicalINode = inode.into();
        physical_inode
            .write_to(buf)
            .map_err(|_| FileIoError::BufferTooSmall)?;
        self.file.seek(SeekFrom::Start(addr.0))?;
        self.file.write(&self.block)?;

        // update bitmap
        let (addr, offset, bit) = self.layout.calc_inode_bitmap_addr(inode_idx)?;
        self.file.seek(SeekFrom::Start(addr.0))?;
        self.file.read(&mut self.block)?;
        self.block[offset] = 1 << bit;
        self.file.seek(SeekFrom::Start(addr.0))?;
        self.file.write(&self.block)?;

        Ok(())
    }

    /// Reads data from the given inode data. Returns the amount of data read.
    pub(crate) fn read(
        &mut self,
        _inode_idx: INodeBlockIndex,
        _read_pos: ReadWritePos,
        _buf: &mut [u8],
    ) -> Result<usize> {
        todo!();
    }

    pub(crate) fn write(
        &mut self,
        inode_idx: INodeBlockIndex,
        write_pos: ReadWritePos,
        buf: &[u8],
    ) -> Result<()> {
        if buf.len() == 0 {
            return Ok(());
        }
        let mut inode = self.read_inode(inode_idx)?;

        // position to start writing the buf
        let buf_write_offset = write_pos.to_file_pos(inode.size)?;

        let required_data_size = buf_write_offset + buf.len();

        // expand file
        if required_data_size > inode.size {
            self.grow_inode_data_size(&mut inode, required_data_size)?;
        }

        todo!();
    }

    fn grow_inode_data_size(&mut self, inode: &mut INode, new_size: FilePos) -> Result<()> {
        if new_size < inode.size {
            return Err(FileIoError::Other(
                "expected new size to be larger than current size",
            ));
        }

        let required_number_of_data_blocks = new_size.0.div_ceil(BLOCK_SIZE as u64);

        for i in 0..required_number_of_data_blocks.min(IMMEDIATE_BLOCK_COUNT as u64) as usize {
            if inode.blocks[i].is_none() {
                inode.blocks[i] = Some(self.allocate_data_block()?);
            }
        }

        if required_number_of_data_blocks > IMMEDIATE_BLOCK_COUNT as u64 {
            todo!();
        }

        Ok(())
    }

    fn read_block(&mut self, addr: FilePos) -> Result<()> {
        self.file.seek(SeekFrom::Start(addr.0))?;
        let read = self.file.read(&mut self.block)?;
        if read != BLOCK_SIZE {
            return Err(FileIoError::IoError(IoError::ReadError));
        }
        Ok(())
    }

    fn write_block(&mut self, addr: FilePos) -> Result<()> {
        self.file.seek(SeekFrom::Start(addr.0))?;
        let written = self.file.write(&mut self.block)?;
        if written != BLOCK_SIZE {
            return Err(FileIoError::IoError(IoError::WriteError));
        }
        Ok(())
    }

    fn allocate_data_block(&mut self) -> Result<DataBlockIndex> {
        let mut block_idx = DataBlockIndex(0);
        for data_block_block_idx in 0..self.layout.data_bitmap_block_count {
            let data_block_bitmap_block_addr = FilePos(
                self.layout.data_bitmap_offset.0
                    + (data_block_block_idx as u64 * BLOCK_SIZE as u64),
            );
            self.read_block(data_block_bitmap_block_addr)?;
            for block_offset in 0..BLOCK_SIZE {
                if self.block[block_offset] != 0xff {
                    let b = self.block[block_offset];
                    for i in 0..8 {
                        if (b >> i) & 1 == 0 {
                            self.block[block_offset] = b | (1 << i);
                            self.write_block(data_block_bitmap_block_addr)?;

                            self.block.fill(0);
                            let addr = self.layout.calc_data_addr(block_idx)?;
                            self.write_block(addr)?;

                            return Ok(block_idx);
                        } else {
                            block_idx.0 += 1;
                        }
                    }
                } else {
                    block_idx.0 += 8;
                }
                if block_idx.0 >= self.layout.data_block_count {
                    return Err(FileIoError::OutOfDiskSpaceError);
                }
            }
        }
        Err(FileIoError::OutOfDiskSpaceError)
    }

    pub(crate) fn create_inode(&mut self, inode: INode) -> Result<INodeBlockIndex> {
        let mut inode_idx: Option<INodeBlockIndex> = None;
        let mut read_pos = self.layout.inode_bitmap_offset;
        let mut byte_offset = 0;
        let mut bit_offset = 0;
        for i in 0..self.layout.inode_count {
            if i.is_multiple_of(PHYSICAL_INODES_PER_BLOCK) {
                self.read_block(read_pos)?;
                read_pos += BLOCK_SIZE;
                byte_offset = 0;
                bit_offset = 0;
            }
            let byte = self.block[byte_offset];
            let bit = (byte >> bit_offset) & 1;
            if bit == 0 {
                inode_idx = Some(INodeBlockIndex(i));
            }
            bit_offset += 1;
            if bit_offset == 8 {
                bit_offset = 0;
                byte_offset += 1;
            }
        }

        if let Some(inode_idx) = inode_idx {
            self.write_inode(inode_idx, inode)?;
            Ok(inode_idx)
        } else {
            Err(FileIoError::Other("out of inodes"))
        }
    }

    fn calc_data_block_idx(
        &self,
        inode: &INode,
        offset: FilePos,
    ) -> Result<Option<DataBlockIndex>> {
        if !(offset.0).is_multiple_of(BLOCK_SIZE as u64) {
            return Err(FileIoError::Other("must be a multiple of block size"));
        }
        // index within the inode block list
        let inode_block_idx = offset.0 / BLOCK_SIZE as u64;

        if inode_block_idx < IMMEDIATE_BLOCK_COUNT as u64 {
            let data_block_idx = inode.blocks[inode_block_idx as usize];
            return Ok(data_block_idx);
        }

        todo!();
    }
}

#[cfg(test)]
mod tests {
    use file_io::TimeSeconds;
    use io::Cursor;
    use myos_api::Uid;

    use crate::physical::BLOCK_SIZE;

    use super::*;

    #[test]
    fn test_root_dir() {
        let mut data = [0; 100 * BLOCK_SIZE];
        let cursor = Cursor::new(&mut data);
        let mut options = FormatVolumeOptions::new(10, 10);
        options.time = TimeSeconds(123);
        let mut fs = format_volume(cursor, options).unwrap();

        let root = fs.root_dir().unwrap();
        assert_eq!(Uid::root(), root.uid());
        assert_eq!(Uid::root(), root.gid());
        assert_eq!(Mode(0o755), root.mode());

        let mut count = 0;
        for entry in root.iter(&mut fs).unwrap() {
            let entry = entry.unwrap();
            assert!(entry.is_dir());
            let dir = entry.to_dir().unwrap();
            assert_eq!(Uid::root(), dir.uid());
            assert_eq!(Uid::root(), dir.gid());
            assert_eq!(Mode(0o755), dir.mode());
            assert_eq!(INodeBlockIndex::root(), dir.inode_idx());

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

        let mut root_dir = fs.root_dir().unwrap();
        let mut file = root_dir
            .create_file(
                &mut fs,
                CreateFileOptions {
                    file_name: "hello.txt",
                    uid: Uid::root(),
                    gid: Uid::root(),
                    mode: Mode(0o755),
                    time: TimeSeconds(123),
                },
            )
            .unwrap();
        file.write_all(b"Hello World!").unwrap();
    }

    // TODO test inode exhaustion
}

pub enum ReadWritePos {
    /// Sets the offset to the provided number of bytes.
    Start(u64),

    /// Sets the offset to the size of this object plus the specified number of
    /// bytes.
    ///
    /// It is possible to seek beyond the end of an object, but it's an error to
    /// seek before byte 0.
    End(i64),
}

impl ReadWritePos {
    pub(crate) fn to_file_pos(&self, size: FilePos) -> Result<FilePos> {
        match self {
            ReadWritePos::Start(v) => Ok(FilePos(*v)),
            ReadWritePos::End(v) => {
                if let Some(v) = size.0.checked_add_signed(*v) {
                    Ok(FilePos(v))
                } else {
                    Err(FileIoError::IoError(IoError::Other("invalid end offset")))
                }
            }
        }
    }
}
