use crate::{INode, INodeId, Result};

pub struct File {
    inode_id: INodeId,
    inode: INode,
}

impl File {
    pub(crate) fn new(inode_id: INodeId, inode: INode) -> Self {
        Self { inode_id, inode }
    }
    
    pub fn write(&mut self, buf: &[u8]) -> Result<usize> {
        todo!();
    }

    pub fn write_all(&mut self, buf: &[u8]) -> Result<()> {
        todo!();
    }

    pub fn flush(&mut self) -> Result<()> {
        todo!();
    }
}

#[cfg(feature = "std")]
impl std::io::Write for File {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        (self as &mut File).write(buf).map_err(|err| err.into())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        (self as &mut File).flush().map_err(|err| err.into())
    }
}
