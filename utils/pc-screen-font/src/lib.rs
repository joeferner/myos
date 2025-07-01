// PC Screen Font
// see https://wiki.osdev.org/PC_Screen_Font
// see https://en.wikipedia.org/wiki/PC_Screen_Font
// see /usr/share/kbd/consolefonts/

#![no_std]

use core::mem::size_of;
use zerocopy::{
    FromBytes, Immutable, KnownLayout, TryFromBytes, Unaligned,
    byteorder::little_endian::{U16, U32},
};

const PSF1_FONT_MAGIC: u16 = 0x0436;
const PSF2_FONT_MAGIC: u32 = 0x864ab572;

/// If this bit is set, the font face will have 512 glyphs. If it is unset, then the font face will have just 256 glyphs.
const PSF1_MODE512: u8 = 0x01;
/// If this bit is set, the font face will have a unicode table.
const PSF1_MODEHASTAB: u8 = 0x02;
/// Equivalent to PSF1_MODEHASTAB
const PSF1_MODESEQ: u8 = 0x04;

/// If this bit is set, the font face will have a unicode table
const PSF2_HAS_UNICODE_TABLE: u32 = 0x00000001;

#[derive(Debug, Clone, Copy)]
pub enum PcFontError {
    InvalidPsfFile,
    // panic!(
    //             "invalid magic, expected 0x{:x} found 0x{:x}",
    //             PSF2_FONT_MAGIC, header.magic
    //         );
    InvalidMagic(u32, u32),
}

#[repr(C, packed)]
#[derive(Debug, Clone, FromBytes, Unaligned, KnownLayout, Immutable)]
struct Psf1Header {
    /// Magic bytes Always 36 04
    pub magic: U16,
    /// PSF Font mode Various font flags, see font modes
    pub mode: u8,
    /// Glyph size in bytes, 8 bit unsigned integer. For psf1, the character size always equals the glyph height
    pub glyph_size: u8,
}

#[repr(C, packed)]
#[derive(Debug, Clone, FromBytes, Unaligned, KnownLayout, Immutable)]
struct Psf2Header {
    /// Always 72 b5 4a 86
    pub magic: U32,
    /// currently always 0
    pub version: U32,
    /// size of the header in bytes (usually 32)
    pub header_size: U32,
    pub flags: U32,
    /// number of glyphs
    pub length: U32,
    /// number of bytes per glyph
    pub glyph_size: U32,
    /// height of each glyph
    pub height: U32,
    /// width of each glyph
    pub width: U32,
}

pub enum FontFormat {
    PSF1,
    PSF2,
}

pub struct Font<'a> {
    format: FontFormat,
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
    pub fn parse(data: &'a [u8]) -> Result<Self, PcFontError> {
        if let Ok(header) = Psf1Header::try_ref_from_bytes(&data[0..size_of::<Psf1Header>()])
            && let Ok(font) = Font::new_from_psf1(header, data)
        {
            return Ok(font);
        }

        if let Ok(header) = Psf2Header::try_ref_from_bytes(&data[0..size_of::<Psf2Header>()]) {
            return Font::new_from_psf2(header, data);
        }

        Err(PcFontError::InvalidPsfFile)
    }

    fn new_from_psf1(header: &Psf1Header, data: &'a [u8]) -> Result<Self, PcFontError> {
        if header.magic != PSF1_FONT_MAGIC {
            return Err(PcFontError::InvalidMagic(
                PSF1_FONT_MAGIC as u32,
                header.magic.get() as u32,
            ));
        }

        let glyph_data_end = if header.mode & PSF1_MODE512 == PSF1_MODE512 {
            size_of::<Psf1Header>() + (512 * header.glyph_size as usize)
        } else {
            size_of::<Psf1Header>() + (256 * header.glyph_size as usize)
        };

        if glyph_data_end > data.len() {
            return Err(PcFontError::InvalidPsfFile);
        }

        let glyph_data = &data[size_of::<Psf1Header>()..glyph_data_end];

        let unicode_table = if header.mode & PSF1_MODEHASTAB == PSF1_MODEHASTAB
            || header.mode & PSF1_MODESEQ == PSF1_MODESEQ
        {
            Some(&data[glyph_data_end..])
        } else {
            None
        };

        Ok(Self {
            format: FontFormat::PSF1,
            glyph_size: header.glyph_size as usize,
            height: header.glyph_size as usize,
            width: 8,
            glyph_data,
            unicode_table,
        })
    }

    fn new_from_psf2(header: &Psf2Header, data: &'a [u8]) -> Result<Self, PcFontError> {
        if header.magic != PSF2_FONT_MAGIC {
            return Err(PcFontError::InvalidMagic(
                PSF2_FONT_MAGIC,
                header.magic.get(),
            ));
        }

        let header_size = header.header_size.get() as usize;
        let glyph_data_end =
            header_size + header.length.get() as usize * header.glyph_size.get() as usize;
        let glyph_data = &data[header_size..glyph_data_end];

        let unicode_table = if (header.flags & PSF2_HAS_UNICODE_TABLE) == PSF2_HAS_UNICODE_TABLE {
            let unicode_table_offset = header_size + glyph_data.len();
            Some(&data[unicode_table_offset..])
        } else {
            None
        };

        Ok(Font {
            format: FontFormat::PSF2,
            glyph_size: header.glyph_size.get() as usize,
            height: header.height.get() as usize,
            width: header.width.get() as usize,
            glyph_data,
            unicode_table,
        })
    }

    pub fn render_char<F>(&self, ch: char, mut f: F)
    where
        F: FnMut(usize, usize, bool),
    {
        let glyph = self.find_glyph(ch);
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

    fn find_glyph(&self, ch: char) -> Option<usize> {
        if let Some(unicode_table) = &self.unicode_table {
            let mut ch_utf8_bytes: [u8; 8] = [0; 8];
            let encoded_len = ch.encode_utf8(&mut ch_utf8_bytes).len();
            let ch = &ch_utf8_bytes[..encoded_len];

            match self.format {
                FontFormat::PSF1 => Font::find_glyph_unicode_table_psf1(unicode_table, ch),
                FontFormat::PSF2 => Font::find_glyph_unicode_table_psf2(unicode_table, ch),
            }
        } else {
            Some(ch as usize)
        }
    }

    fn find_glyph_unicode_table_psf1(unicode_table: &[u8], ch: &[u8]) -> Option<usize> {
        // TODO handle found_fffe, multiple unicode characters can exist in a single entry
        // fffe denotes this
        let mut _found_fffe = false;
        let mut glyph_idx = 0;
        let mut it = unicode_table.iter();
        loop {
            if let Some(low) = it.next()
                && let Some(high) = it.next()
            {
                if *high == 0xff && *low == 0xff {
                    glyph_idx += 1;
                    continue;
                }

                if *high == 0xff && *low == 0xfe {
                    _found_fffe = true;
                }

                if ch.len() == 1 && *high == 0x00 && ch[0] == *low {
                    return Some(glyph_idx);
                }
            } else {
                return None;
            }
        }
    }

    fn find_glyph_unicode_table_psf2(unicode_table: &[u8], ch: &[u8]) -> Option<usize> {
        for (glyph_idx, code) in unicode_table.split(|&v| v == 0xff).enumerate() {
            if code == ch {
                return Some(glyph_idx);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TAMSYN_PSF1: &[u8] = include_bytes!("./resources/test/Tamsyn8x16b.psf1");
    const TAMSYN_NOTABLE_PSF1: &[u8] = include_bytes!("./resources/test/Tamsyn8x16b.notable.psf1");
    const TAMSYN_PSF2: &[u8] = include_bytes!("./resources/test/Tamsyn8x16b.psf2");
    const TAMSYN_NOTABLE_PSF2: &[u8] = include_bytes!("./resources/test/Tamsyn8x16b.notable.psf2");

    fn render_char_to_buffer(font: &Font, ch: char, stride: usize, buffer: &mut [u8]) {
        font.render_char(ch, |x, y, v| {
            let offset = y * stride + x;
            buffer[offset] = if v { 1 } else { 0 };
        });
    }

    macro_rules! test_psf {
        ($font_data:expr) => {
            let mut buffer: [u8; 16 * 8] = [0; 16 * 8];
            let font = Font::parse($font_data).unwrap();
            assert_eq!(font.height, 16);
            assert_eq!(font.width, 8);
            render_char_to_buffer(&font, 'R', 8, &mut buffer);
            let expected: [u8; 16 * 8] = [
                0, 0, 0, 0, 0, 0, 0, 0, // 0
                0, 0, 0, 0, 0, 0, 0, 0, // 1
                0, 0, 0, 0, 0, 0, 0, 0, // 2
                0, 1, 1, 1, 1, 1, 0, 0, // 3
                0, 1, 1, 0, 0, 1, 1, 0, // 4
                0, 1, 1, 0, 0, 1, 1, 0, // 5
                0, 1, 1, 0, 0, 1, 1, 0, // 6
                0, 1, 1, 1, 1, 1, 0, 0, // 7
                0, 1, 1, 0, 1, 1, 0, 0, // 8
                0, 1, 1, 0, 0, 1, 1, 0, // 9
                0, 1, 1, 0, 0, 1, 1, 0, // 10
                0, 1, 1, 0, 0, 1, 1, 0, // 11
                0, 0, 0, 0, 0, 0, 0, 0, // 12
                0, 0, 0, 0, 0, 0, 0, 0, // 13
                0, 0, 0, 0, 0, 0, 0, 0, // 14
                0, 0, 0, 0, 0, 0, 0, 0, // 15
            ];
            assert_eq!(buffer, expected);
        };
    }

    #[test]
    fn test_psf1_without_unicode_table() {
        test_psf!(TAMSYN_NOTABLE_PSF1);
    }

    #[test]
    fn test_psf1_with_unicode_table() {
        test_psf!(TAMSYN_PSF1);
    }

    #[test]
    fn test_psf2_without_unicode_table() {
        test_psf!(TAMSYN_NOTABLE_PSF2);
    }

    #[test]
    fn test_psf2_with_unicode_table() {
        test_psf!(TAMSYN_PSF2);
    }
}
