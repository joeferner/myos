#![no_std]
#![feature(assert_matches)]

use core::num::ParseIntError;

pub mod colors;

macro_rules! next_value {
    ($it:expr) => {
        if let Some(val) = $it.next()
            && let Ok(val) = val
        {
            val
        } else {
            return Err(());
        }
    };
}

macro_rules! assert_no_more_items {
    ($it:expr) => {
        if let Some(_) = $it.next() {
            return Err(());
        }
    };
}

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
    /// moves cursor to home position (0, 0)
    CursorHome,
    /// moves cursor to line #, column #
    CursorTo(u8, u8),
    /// moves cursor up # lines
    CursorUp(u8),
    /// moves cursor down # lines
    CursorDown(u8),
    /// moves cursor right # columns
    CursorRight(u8),
    /// moves cursor left # columns
    CursorLeft(u8),
    /// reset all modes (styles and colors)
    ResetAllModes,
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
            let event = self.try_parse_cursor(rest)?;
            if !matches!(event, AnsiEvent::None) {
                self.buffer.clear();
                return Ok(event);
            }

            let event = self.try_parse_graphics_mode(rest)?;
            if !matches!(event, AnsiEvent::None) {
                self.buffer.clear();
                return Ok(event);
            }
        }

        Ok(AnsiEvent::None)
    }

    fn try_parse_cursor(&self, s: &str) -> Result<AnsiEvent, ()> {
        if s == "H" {
            return Ok(AnsiEvent::CursorHome);
        }

        // ESC[{line};{column}H
        if s.ends_with("H") || s.ends_with("f") {
            let s = &s[0..s.len() - 1];
            let mut it = s.split(";").map(|v| v.parse::<u8>());

            let line: u8 = next_value!(it);
            let column: u8 = next_value!(it);
            assert_no_more_items!(it);
            return Ok(AnsiEvent::CursorTo(line, column));
        }

        if s.ends_with("A") || s.ends_with("B") || s.ends_with("C") || s.ends_with("D") {
            if let Ok(val) = s[0..s.len() - 1].parse::<u8>() {
                if s.ends_with("A") {
                    return Ok(AnsiEvent::CursorUp(val));
                } else if s.ends_with("B") {
                    return Ok(AnsiEvent::CursorDown(val));
                } else if s.ends_with("C") {
                    return Ok(AnsiEvent::CursorRight(val));
                } else if s.ends_with("D") {
                    return Ok(AnsiEvent::CursorLeft(val));
                } else {
                    return Err(());
                }
            } else {
                return Err(());
            }
        }

        Ok(AnsiEvent::None)
    }

    /// see https://gist.github.com/fnky/458719343aabd01cfb17a3a4f7296797#rgb-colors
    ///
    /// ESC[38;2;{r};{g};{b}m  Set foreground color as RGB.
    /// ESC[48;2;{r};{g};{b}m  Set background color as RGB.
    ///
    fn try_parse_graphics_mode(&self, s: &str) -> Result<AnsiEvent, ()> {
        if !s.ends_with("m") {
            return Ok(AnsiEvent::None);
        }
        let s = &s[0..s.len() - 1];

        let mut it = s.split(";").map(|v| v.parse::<u8>());

        // 0  - reset all modes (styles and colors)
        // 38 - set forground
        // 48 - set background
        let code: u8 = next_value!(it);

        if code == 0 {
            Ok(AnsiEvent::ResetAllModes)
        } else if code == 38 || code == 48 {
            self.try_parse_graphics_color(code, &mut it)
        } else {
            Err(())
        }
    }

    fn try_parse_graphics_color<T>(&self, code: u8, it: &mut T) -> Result<AnsiEvent, ()>
    where
        T: Iterator<Item = Result<u8, ParseIntError>>,
    {
        let mut color = Color {
            red: 0,
            green: 0,
            blue: 0,
        };

        // 2 - rgb color, 5 - 256 colors
        let mode: u8 = next_value!(it);

        if mode == 2 {
            color.red = next_value!(it);
            color.green = next_value!(it);
            color.blue = next_value!(it);
            assert_no_more_items!(it);
        } else if mode == 5 {
            let id = next_value!(it);
            color = colors::COLORS[id as usize];
            assert_no_more_items!(it);
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

    macro_rules! test_single_event {
        ($s:expr) => {{
            let mut parser = AnsiEscapeParser::new();
            let events: Vec<AnsiEvent> = push_str(&mut parser, $s).collect();
            assert_eq!(1, events.len());
            events
        }};
    }

    #[test]
    pub fn test_cursor_home() {
        let events = test_single_event!("\u{1b}[H");
        assert_matches!(events[0], AnsiEvent::CursorHome);
    }

    #[test]
    pub fn test_cursor_to() {
        let events = test_single_event!("\u{1b}[10;20H");
        assert_matches!(events[0], AnsiEvent::CursorTo(10, 20));

        let events = test_single_event!("\u{1b}[10;20f");
        assert_matches!(events[0], AnsiEvent::CursorTo(10, 20));
    }

    #[test]
    pub fn test_cursor_up() {
        let events = test_single_event!("\u{1b}[5A");
        assert_matches!(events[0], AnsiEvent::CursorUp(5));
    }

    #[test]
    pub fn test_cursor_down() {
        let events = test_single_event!("\u{1b}[5B");
        assert_matches!(events[0], AnsiEvent::CursorDown(5));
    }

    #[test]
    pub fn test_cursor_right() {
        let events = test_single_event!("\u{1b}[5C");
        assert_matches!(events[0], AnsiEvent::CursorRight(5));
    }

    #[test]
    pub fn test_cursor_left() {
        let events = test_single_event!("\u{1b}[5D");
        assert_matches!(events[0], AnsiEvent::CursorLeft(5));
    }

    #[test]
    pub fn test_reset_all_modes() {
        let events = test_single_event!("\u{1b}[0m");
        assert_matches!(events[0], AnsiEvent::ResetAllModes);
    }

    #[test]
    pub fn test_color_by_id() {
        let events = test_single_event!("\u{1b}[38;5;177m");
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
        let events = test_single_event!("\u{1b}[38;2;255;0;50m");
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
        let events = test_single_event!("\u{1b}[38;2;500;0;50m");
        assert_invalid_sequence(&events[0], "\u{1b}[38;2;500;0;50m");
    }

    #[test]
    pub fn test_rgb_color_value_too_many_args() {
        let events = test_single_event!("\u{1b}[38;2;500;0;50;12m");
        assert_invalid_sequence(&events[0], "\u{1b}[38;2;500;0;50;12m");
    }

    #[test]
    pub fn test_rgb_color_value_too_few_args() {
        let events = test_single_event!("\u{1b}[38;2;500;0m");
        assert_invalid_sequence(&events[0], "\u{1b}[38;2;500;0m");
    }
}
