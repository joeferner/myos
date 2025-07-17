pub trait Read {}

pub trait Write {}

pub trait Seek {}

#[cfg(feature = "std")]
impl Read for std::fs::File {}

#[cfg(feature = "std")]
impl Write for std::fs::File {}

#[cfg(feature = "std")]
impl Seek for std::fs::File {}

/// A sum of `Read`, `Write` and `Seek` traits.
pub trait ReadWriteSeek: Read + Write + Seek {}
impl<T: Read + Write + Seek> ReadWriteSeek for T {}

pub struct Cursor<'a> {
    data: &'a [u8],
}

impl<'a> Cursor<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data }
    }
}

impl<'a> Read for Cursor<'a> {}

impl<'a> Write for Cursor<'a> {}

impl<'a> Seek for Cursor<'a> {}
