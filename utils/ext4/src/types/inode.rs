use file_io::{FilePos, Result};
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

use crate::source::Ext4Source;

#[repr(C, packed)]
#[derive(Clone, IntoBytes, FromBytes, Immutable, KnownLayout)]
pub(crate) struct INode {}

impl INode {
    pub(crate) fn read<T: Ext4Source>(source: &T, file_pos: &FilePos) -> Result<Self> {}
}
