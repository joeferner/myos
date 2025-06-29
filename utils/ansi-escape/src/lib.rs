#![no_std]

const BUFFER_SIZE: usize = 20;
const ESCAPE: char = '\x1b';

macro_rules! try_parse_char {
    ($self:expr, $offset:expr, $ch:expr) => {
        if $offset >= $self.buffer_len {
            return AnsiEvent::None;
        }
        if $self.buffer[$offset] != $ch {
            return AnsiEvent::None;
        }
        $offset += 1;
    };
}

macro_rules! try_parse_u16 {
    ($self:expr, $offset:expr) => {{
        let mut i = $offset;
        let end_offset = loop {
            if i >= $self.buffer_len {
                break i - 1;
            }
            if $self.buffer[i] < '0' && $self.buffer[i] > '9' {
                break i - 1;
            }
            i += 1;
        };
        if end_offset <= $offset {
            return AnsiEvent::None;
        }
        let ret = chartoi::chartou::<u16>(&$self.buffer[$offset..end_offset]);
        if ret.is_ok() {
            $offset = end_offset;
        }
        ret
    }};
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnsiEvent {
    None,
    InvalidEscapeSequence([char; BUFFER_SIZE], usize),
    SetForegroundColor(Color),
}

pub struct AnsiEscapeParser {
    buffer: [char; BUFFER_SIZE],
    buffer_len: usize,
}

impl AnsiEscapeParser {
    pub fn new() -> Self {
        Self {
            buffer: ['\0'; BUFFER_SIZE],
            buffer_len: 0,
        }
    }

    pub fn push(&mut self, ch: char) -> AnsiEvent {
        if ch == ESCAPE || self.buffer_len > 0 {
            if !self.push_buffer(ch) {
                return AnsiEvent::InvalidEscapeSequence(self.buffer, self.buffer_len);
            }
            self.parse_buffer()
        } else {
            AnsiEvent::None
        }
    }

    fn push_buffer(&mut self, ch: char) -> bool {
        if self.buffer_len >= self.buffer.len() {
            return false;
        }
        self.buffer[self.buffer_len] = ch;
        self.buffer_len += 1;
        true
    }

    fn parse_buffer(&mut self) -> AnsiEvent {
        let mut offset = 0;
        try_parse_char!(self, offset, ESCAPE);
        try_parse_char!(self, offset, '[');

        let event = self.try_parse_set_rgb_color(offset);
        if !matches!(event, AnsiEvent::None) {
            self.buffer_len = 0;
            return event;
        }
        AnsiEvent::None
    }

    /// see https://gist.github.com/fnky/458719343aabd01cfb17a3a4f7296797#rgb-colors
    ///
    /// ESC[38;2;{r};{g};{b}m  Set foreground color as RGB.
    /// ESC[48;2;{r};{g};{b}m  Set background color as RGB.
    ///
    fn try_parse_set_rgb_color(&self, mut offset: usize) -> AnsiEvent {
        let code = if let Ok(code) = try_parse_u16!(self, offset) {
            if code != 38 && code != 48 {
                return AnsiEvent::None;
            } else {
                code
            }
        } else {
            return AnsiEvent::None;
        };

        try_parse_char!(self, offset, ';');
        
        AnsiEvent::SetForegroundColor(Color {
            red: 0,
            green: 0,
            blue: 0,
        })
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use super::*;
    use std::vec::Vec;

    fn push_str(parser: &mut AnsiEscapeParser, s: &str) -> impl Iterator<Item = AnsiEvent> {
        s.chars()
            .map(|ch| parser.push(ch))
            .filter(|e| !matches!(e, AnsiEvent::None))
    }

    #[test]
    pub fn test_rgb_color() {
        let mut parser = AnsiEscapeParser::new();
        let events: Vec<AnsiEvent> = push_str(&mut parser, "\x1b[38;2;255;0;50m").collect();
        assert_eq!(1, events.len());
    }
}
