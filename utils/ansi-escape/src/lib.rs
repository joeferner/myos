#![no_std]

const BUFFER_SIZE: usize = 20;
const ESCAPE: char = '\x1b';

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
        if let Some(event) = self.try_parse_set_rgb_color() {
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
    fn try_parse_set_rgb_color(&self) -> Option<AnsiEvent> {
        try_parse_escape_start!();
        None
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
