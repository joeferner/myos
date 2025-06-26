// PC Screen Font
// see https://wiki.osdev.org/PC_Screen_Font
// see /usr/share/kbd/consolefonts/

pub const DEFAULT_8X16: &[u8] = include_bytes!("default8x16.psfu");

const PSF2_FONT_MAGIC: u32 = 0x72b54a86;

/// If this bit is set, the font face will have a unicode table
const PSF2_HAS_UNICODE_TABLE: u32 = 0x00000001;

#[repr(C)]
struct Psf2Header {
    /// Always 72 b5 4a 86
    pub magic: u32,
    /// currently always 0
    pub version: u32,
    /// size of the header in bytes (usually 32)
    pub header_size: u32,
    pub flags: u32,
    /// number of glyphs
    pub length: u32,
    /// number of bytes per glyph
    pub glyph_size: u32,
    /// height of each glyph
    pub height: u32,
    /// width of each glyph
    pub width: u32,
}

pub struct Font {}

impl Font {
    pub fn new(data: &[u8]) -> Self {
        let (_head, header, _tail) = unsafe { data.align_to::<Psf2Header>() };
        let header = header[0];
        if header.magic != PSF2_FONT_MAGIC {
            panic!(
                "invalid magic, expected {:x} found {:x}",
                PSF2_FONT_MAGIC, header.magic
            );
        }

        Font {}
    }
}
