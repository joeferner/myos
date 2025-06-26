// PC Screen Font
// see https://wiki.osdev.org/PC_Screen_Font
// see https://en.wikipedia.org/wiki/PC_Screen_Font
// see /usr/share/kbd/consolefonts/

#[repr(align(4))]
pub struct FontData([u8; include_bytes!("default8x16.psfu").len()]);

pub const DEFAULT_8X16: &FontData = &FontData(*include_bytes!("default8x16.psfu"));

const PSF2_FONT_MAGIC: u32 = 0x864ab572;

/// If this bit is set, the font face will have a unicode table
const PSF2_HAS_UNICODE_TABLE: u32 = 0x00000001;

#[repr(C)]
#[derive(Debug, Clone)]
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

pub struct Font<'a> {
    /// number of bytes per glyph
    glyph_size: usize,
    /// height of each glyph
    pub height: usize,
    /// width of each glyph
    pub width: usize,
    glyph_data: &'a [u8],
    unicode_table: Option<&'a [u8]>,
}

impl<'a> Font<'a> {
    pub fn new(data: &'a FontData) -> Self {
        let data = &data.0;
        let header_data = &data[0..core::mem::size_of::<Psf2Header>()];
        let header = unsafe { &(*(header_data.as_ptr() as *const Psf2Header)) };
        if header.magic != PSF2_FONT_MAGIC {
            panic!(
                "invalid magic, expected 0x{:x} found 0x{:x}",
                PSF2_FONT_MAGIC, header.magic
            );
        }

        let header_size = header.header_size as usize;
        let glyph_data_end = header_size + header.length as usize * header.glyph_size as usize;
        let glyph_data = &data[header_size..glyph_data_end];

        let unicode_table = if (header.flags & PSF2_HAS_UNICODE_TABLE) == PSF2_HAS_UNICODE_TABLE {
            let unicode_table_offset = header_size + glyph_data.len();
            Some(&data[unicode_table_offset..])
        } else {
            None
        };

        Font {
            glyph_size: header.glyph_size as usize,
            height: header.height as usize,
            width: header.width as usize,
            glyph_data,
            unicode_table,
        }
    }

    pub fn render_char<F>(&self, ch: char, mut f: F)
    where
        F: FnMut(usize, usize, bool),
    {
        let ch_str = ch.to_string();
        let ch_utf8_bytes: &[u8] = ch_str.as_bytes();
        let glyph = self.find_glyph(ch_utf8_bytes);
        if let Some(glyph) = glyph {
            let glyph_offset = glyph * self.glyph_size;
            let glyph_end = glyph_offset + self.glyph_size;
            let mut glyph_it = self.glyph_data[glyph_offset..glyph_end].iter();
            let mut glyph_shift = 7;
            let mut cur = glyph_it.next();
            for y in 0..self.height {
                for x in 0..self.width {
                    if let Some(cur) = cur {
                        f(x, y, ((cur >> glyph_shift) & 1) == 1);
                    }
                    glyph_shift -= 1;
                    if glyph_shift < 0 {
                        glyph_shift = 7;
                        cur = glyph_it.next();
                    }
                }
                // skip padding
                if glyph_shift != 7 {
                    glyph_shift = 7;
                    cur = glyph_it.next();
                }
            }
        }
    }

    fn find_glyph(&self, ch: &[u8]) -> Option<usize> {
        if let Some(unicode_table) = &self.unicode_table {
            Font::find_glyph_unicode_table(unicode_table, ch)
        } else {
            panic!("unicode_table not present");
        }
    }

    fn find_glyph_unicode_table(unicode_table: &[u8], ch: &[u8]) -> Option<usize> {
        let mut glyph_idx = 0;
        let mut ch_idx = 0;
        let mut unicode_table_idx = 0;

        // TODO change to iterable
        while unicode_table_idx < unicode_table.len() {
            let entry = unicode_table[unicode_table_idx];
            if entry == 0xff {
                if ch_idx == ch.len() {
                    return Some(glyph_idx);
                }
                glyph_idx += 1;
                unicode_table_idx += 1;
                ch_idx = 0;
            } else if ch_idx < ch.len() && entry == ch[ch_idx] {
                unicode_table_idx += 1;
                ch_idx += 1;
            } else {
                // no match, skip this entry
                while unicode_table_idx < unicode_table.len() {
                    if unicode_table[unicode_table_idx] == 0xff {
                        break;
                    }
                    unicode_table_idx += 1;
                }
            }
        }
        None
    }
}
