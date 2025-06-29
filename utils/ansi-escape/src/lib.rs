#![no_std]
#![feature(assert_matches)]

pub mod colors;

const BUFFER_SIZE: usize = 20;
const ESCAPE: char = '\u{1b}';

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnsiEvent {
    None,
    Char(char),
    InvalidEscapeSequence(heapless::String<BUFFER_SIZE>),
    SetForegroundColor(Color),
    SetBackgroundColor(Color),
}

pub struct AnsiEscapeParser {
    buffer: heapless::String<BUFFER_SIZE>,
}

impl AnsiEscapeParser {
    pub fn new() -> Self {
        Self {
            buffer: heapless::String::new(),
        }
    }

    pub fn push(&mut self, ch: char) -> AnsiEvent {
        if ch == ESCAPE || self.buffer.len() > 0 {
            // this check should not be need since we should have failed and cleared last push
            // but to be safe we'll keep in here
            if let Err(_) = self.buffer.push(ch) {
                let result = AnsiEvent::InvalidEscapeSequence(self.buffer.clone());
                self.buffer.clear();
                return result;
            }
            if self.buffer.len() >= self.buffer.capacity() {
                let result = AnsiEvent::InvalidEscapeSequence(self.buffer.clone());
                self.buffer.clear();
                return result;
            }
            if let Ok(event) = self.parse_buffer() {
                event
            } else {
                let result = AnsiEvent::InvalidEscapeSequence(self.buffer.clone());
                self.buffer.clear();
                result
            }
        } else {
            AnsiEvent::Char(ch)
        }
    }

    fn parse_buffer(&mut self) -> Result<AnsiEvent, ()> {
        if !self.buffer.starts_with("\u{1b}[") {
            return Ok(AnsiEvent::None);
        }

        if let Some(rest) = self.buffer.get(2..) {
            let event = self.try_parse_set_color(rest)?;
            if !matches!(event, AnsiEvent::None) {
                self.buffer.clear();
                return Ok(event);
            }
        }

        Ok(AnsiEvent::None)
    }

    /// see https://gist.github.com/fnky/458719343aabd01cfb17a3a4f7296797#rgb-colors
    ///
    /// ESC[38;2;{r};{g};{b}m  Set foreground color as RGB.
    /// ESC[48;2;{r};{g};{b}m  Set background color as RGB.
    ///
    fn try_parse_set_color(&self, s: &str) -> Result<AnsiEvent, ()> {
        if !s.ends_with("m") {
            return Ok(AnsiEvent::None);
        }
        let s = &s[0..s.len() - 1];

        // 38 - set forground, 48 - set background
        let code: u8;

        // 2 - rgb color, 5 - 256 colors
        let mode: u8;

        let mut color = Color {
            red: 0,
            green: 0,
            blue: 0,
        };

        let mut it = s.split(";").map(|v| v.parse::<u8>());

        // code
        if let Some(val) = it.next()
            && let Ok(val) = val
        {
            code = val;
        } else {
            return Err(());
        }

        // mode
        if let Some(val) = it.next()
            && let Ok(val) = val
        {
            mode = val;
        } else {
            return Err(());
        }

        if mode == 2 {
            // red
            if let Some(val) = it.next()
                && let Ok(val) = val
            {
                color.red = val;
            } else {
                return Err(());
            }

            // green
            if let Some(val) = it.next()
                && let Ok(val) = val
            {
                color.green = val;
            } else {
                return Err(());
            }

            // blue
            if let Some(val) = it.next()
                && let Ok(val) = val
            {
                color.blue = val;
            } else {
                return Err(());
            }

            if let Some(_) = it.next() {
                return Err(());
            }
        } else if mode == 5 {
            // id
            if let Some(val) = it.next()
                && let Ok(val) = val
            {
                color = colors::COLORS[val as usize];
            } else {
                return Err(());
            }

            if let Some(_) = it.next() {
                return Err(());
            }
        } else {
            return Err(());
        };

        if code == 38 {
            Ok(AnsiEvent::SetForegroundColor(color))
        } else if code == 48 {
            Ok(AnsiEvent::SetBackgroundColor(color))
        } else {
            Err(())
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use super::*;
    use core::assert_matches::assert_matches;
    use std::vec::Vec;

    fn push_str(parser: &mut AnsiEscapeParser, s: &str) -> impl Iterator<Item = AnsiEvent> {
        s.chars()
            .map(|ch| parser.push(ch))
            .filter(|e| !matches!(e, AnsiEvent::None))
    }

    fn assert_invalid_sequence(event: &AnsiEvent, expected_str: &str) {
        if let AnsiEvent::InvalidEscapeSequence(val) = &event {
            assert_eq!(val, expected_str);
        } else {
            panic!(
                "expected AnsiEvent::InvalidEscapeSequence, found {:?}",
                event
            );
        }
    }

    #[test]
    pub fn test_color_by_id() {
        let mut parser = AnsiEscapeParser::new();
        let events: Vec<AnsiEvent> = push_str(&mut parser, "\u{1b}[38;5;177m").collect();
        assert_eq!(1, events.len());
        assert_matches!(
            events[0],
            AnsiEvent::SetForegroundColor(Color {
                red: 215,
                green: 135,
                blue: 255
            })
        );
    }

    #[test]
    pub fn test_rgb_color() {
        let mut parser = AnsiEscapeParser::new();
        let events: Vec<AnsiEvent> = push_str(&mut parser, "\u{1b}[38;2;255;0;50m").collect();
        assert_eq!(1, events.len());
        assert_matches!(
            events[0],
            AnsiEvent::SetForegroundColor(Color {
                red: 255,
                green: 0,
                blue: 50
            })
        );
    }

    #[test]
    pub fn test_rgb_color_value_too_large() {
        let mut parser = AnsiEscapeParser::new();
        let events: Vec<AnsiEvent> = push_str(&mut parser, "\u{1b}[38;2;500;0;50m").collect();
        assert_eq!(1, events.len());
        assert_invalid_sequence(&events[0], "\u{1b}[38;2;500;0;50m");
    }

    #[test]
    pub fn test_rgb_color_value_too_many_args() {
        let mut parser = AnsiEscapeParser::new();
        let events: Vec<AnsiEvent> = push_str(&mut parser, "\u{1b}[38;2;500;0;50;12m").collect();
        assert_eq!(1, events.len());
        assert_invalid_sequence(&events[0], "\u{1b}[38;2;500;0;50;12m");
    }

    #[test]
    pub fn test_rgb_color_value_too_few_args() {
        let mut parser = AnsiEscapeParser::new();
        let events: Vec<AnsiEvent> = push_str(&mut parser, "\u{1b}[38;2;500;0m").collect();
        assert_eq!(1, events.len());
        assert_invalid_sequence(&events[0], "\u{1b}[38;2;500;0m");
    }
}
