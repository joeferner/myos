#![no_std]
#![feature(assert_matches)]

use core::{fmt::Display, num::ParseIntError};

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

impl Color {
    pub const fn rgb(red: u8, green: u8, blue: u8) -> Color {
        Color { red, green, blue }
    }

    pub const fn white() -> Color {
        Color::rgb(255, 255, 255)
    }

    pub const fn black() -> Color {
        Color::rgb(0, 0, 0)
    }

    pub const fn red() -> Color {
        Color::rgb(255, 0, 0)
    }

    pub const fn green() -> Color {
        Color::rgb(0, 255, 0)
    }

    pub const fn blue() -> Color {
        Color::rgb(0, 0, 255)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ansi {
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
    Bold,
    ResetBold,
    Char(char),
    ForegroundColor(Color),
    BackgroundColor(Color),
    DefaultForeground,
    DefaultBackground,
}

impl Display for Ansi {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Ansi::CursorHome => write!(f, "\u{1b}[H"),
            Ansi::CursorTo(line, column) => write!(f, "\u{1b}[{line};{column}H"),
            Ansi::CursorUp(n) => write!(f, "\u{1b}[{n}A"),
            Ansi::CursorDown(n) => write!(f, "\u{1b}[{n}B"),
            Ansi::CursorRight(n) => write!(f, "\u{1b}[{n}C"),
            Ansi::CursorLeft(n) => write!(f, "\u{1b}[{n}D"),
            Ansi::ResetAllModes => write!(f, "\u{1b}[0m"),
            Ansi::Bold => write!(f, "\u{1b}[1m"),
            Ansi::ResetBold => write!(f, "\u{1b}[22m"),
            Ansi::Char(ch) => write!(f, "{ch}"),
            Ansi::ForegroundColor(color) => write!(
                f,
                "\u{1b}[38;2;{};{};{}m",
                color.red, color.green, color.blue
            ),
            Ansi::BackgroundColor(color) => write!(
                f,
                "\u{1b}[48;2;{};{};{}m",
                color.red, color.green, color.blue
            ),
            Ansi::DefaultForeground => write!(f, "\u{1b}[39m"),
            Ansi::DefaultBackground => write!(f, "\u{1b}[49m"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum AnsiEscapeParserError {
    InvalidEscapeSequence(heapless::String<BUFFER_SIZE>),
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

    pub fn push(&mut self, ch: char) -> Result<Option<Ansi>, AnsiEscapeParserError> {
        if ch == ESCAPE || !self.buffer.is_empty() {
            // this check should not be need since we should have failed and cleared last push
            // but to be safe we'll keep in here
            if self.buffer.push(ch).is_err() {
                let result = Err(AnsiEscapeParserError::InvalidEscapeSequence(
                    self.buffer.clone(),
                ));
                self.buffer.clear();
                return result;
            }
            if self.buffer.len() >= self.buffer.capacity() {
                let result = Err(AnsiEscapeParserError::InvalidEscapeSequence(
                    self.buffer.clone(),
                ));
                self.buffer.clear();
                return result;
            }
            if let Ok(event) = self.parse_buffer() {
                Ok(event)
            } else {
                let result = Err(AnsiEscapeParserError::InvalidEscapeSequence(
                    self.buffer.clone(),
                ));
                self.buffer.clear();
                result
            }
        } else {
            Ok(Some(Ansi::Char(ch)))
        }
    }

    fn parse_buffer(&mut self) -> Result<Option<Ansi>, ()> {
        if !self.buffer.starts_with("\u{1b}[") {
            return Ok(None);
        }

        if let Some(rest) = self.buffer.get(2..) {
            let event = self.try_parse_cursor(rest)?;
            if event.is_some() {
                self.buffer.clear();
                return Ok(event);
            }

            let event = self.try_parse_graphics_mode(rest)?;
            if event.is_some() {
                self.buffer.clear();
                return Ok(event);
            }
        }

        Ok(None)
    }

    fn try_parse_cursor(&self, s: &str) -> Result<Option<Ansi>, ()> {
        if s == "H" {
            return Ok(Some(Ansi::CursorHome));
        }

        // ESC[{line};{column}H
        if s.ends_with("H") || s.ends_with("f") {
            let s = &s[0..s.len() - 1];
            let mut it = s.split(";").map(|v| v.parse::<u8>());

            let line: u8 = next_value!(it);
            let column: u8 = next_value!(it);
            assert_no_more_items!(it);
            return Ok(Some(Ansi::CursorTo(line, column)));
        }

        if s.ends_with("A") || s.ends_with("B") || s.ends_with("C") || s.ends_with("D") {
            if let Ok(val) = s[0..s.len() - 1].parse::<u8>() {
                if s.ends_with("A") {
                    return Ok(Some(Ansi::CursorUp(val)));
                } else if s.ends_with("B") {
                    return Ok(Some(Ansi::CursorDown(val)));
                } else if s.ends_with("C") {
                    return Ok(Some(Ansi::CursorRight(val)));
                } else if s.ends_with("D") {
                    return Ok(Some(Ansi::CursorLeft(val)));
                } else {
                    return Err(());
                }
            } else {
                return Err(());
            }
        }

        Ok(None)
    }

    /// see https://gist.github.com/fnky/458719343aabd01cfb17a3a4f7296797#rgb-colors
    ///
    /// ESC[38;2;{r};{g};{b}m  Set foreground color as RGB.
    /// ESC[48;2;{r};{g};{b}m  Set background color as RGB.
    ///
    fn try_parse_graphics_mode(&self, s: &str) -> Result<Option<Ansi>, ()> {
        if !s.ends_with("m") {
            return Ok(None);
        }
        let s = &s[0..s.len() - 1];

        let mut it = s.split(";").map(|v| v.parse::<u8>());

        // 0  - reset all modes (styles and colors)
        // 1  - set bold mode
        // 22 - reset bold mode
        // 38 - set forground
        // 39 - default foreground
        // 48 - set background
        // 49 - default background
        let code: u8 = next_value!(it);

        if code == 0 {
            Ok(Some(Ansi::ResetAllModes))
        } else if code == 1 {
            Ok(Some(Ansi::Bold))
        } else if code == 22 {
            Ok(Some(Ansi::ResetBold))
        } else if code == 38 || code == 48 {
            self.try_parse_graphics_color(code, &mut it)
        } else if code == 39 {
            assert_no_more_items!(it);
            Ok(Some(Ansi::DefaultForeground))
        } else if code == 49 {
            assert_no_more_items!(it);
            Ok(Some(Ansi::DefaultBackground))
        } else {
            Err(())
        }
    }

    fn try_parse_graphics_color<T>(&self, code: u8, it: &mut T) -> Result<Option<Ansi>, ()>
    where
        T: Iterator<Item = Result<u8, ParseIntError>>,
    {
        let mut color = Color::black();

        // 2  - rgb color
        // 5  - 256 colors
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
            Ok(Some(Ansi::ForegroundColor(color)))
        } else if code == 48 {
            Ok(Some(Ansi::BackgroundColor(color)))
        } else {
            Err(())
        }
    }
}

impl Default for AnsiEscapeParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use super::*;
    use core::assert_matches::assert_matches;
    use std::vec::Vec;

    fn push_str(parser: &mut AnsiEscapeParser, s: &str) -> impl Iterator<Item = Ansi> {
        s.chars()
            .map(|ch| parser.push(ch).unwrap())
            .filter(|e| e.is_some())
            .map(|e| e.unwrap())
    }

    macro_rules! test_single_event {
        ($s:expr) => {{
            let mut parser = AnsiEscapeParser::new();
            let events: Vec<Ansi> = push_str(&mut parser, $s).collect();
            assert_eq!(1, events.len());
            events
        }};
    }

    macro_rules! test_single_invalid_sequence {
        ($s:expr, $expected:expr) => {
            let mut parser = AnsiEscapeParser::new();
            let mut found_err = false;
            for ch in $s.chars() {
                match parser.push(ch) {
                    Ok(event) => {
                        if event.is_some() {
                            panic!("expected only invalid sequence but found {:?}", event)
                        }
                    }
                    Err(err) => {
                        if found_err {
                            panic!("already found error but got {:?}", err);
                        }
                        match err {
                            AnsiEscapeParserError::InvalidEscapeSequence(seq) => {
                                assert_eq!($expected, seq);
                            }
                        }
                        found_err = true;
                    }
                }
            }
            if !found_err {
                panic!("never found error");
            }
        };
    }

    #[test]
    pub fn test_cursor_home() {
        let events = test_single_event!("\u{1b}[H");
        assert_matches!(events[0], Ansi::CursorHome);
    }

    #[test]
    pub fn test_cursor_to() {
        let events = test_single_event!("\u{1b}[10;20H");
        assert_matches!(events[0], Ansi::CursorTo(10, 20));

        let events = test_single_event!("\u{1b}[10;20f");
        assert_matches!(events[0], Ansi::CursorTo(10, 20));
    }

    #[test]
    pub fn test_cursor_up() {
        let events = test_single_event!("\u{1b}[5A");
        assert_matches!(events[0], Ansi::CursorUp(5));
    }

    #[test]
    pub fn test_cursor_down() {
        let events = test_single_event!("\u{1b}[5B");
        assert_matches!(events[0], Ansi::CursorDown(5));
    }

    #[test]
    pub fn test_cursor_right() {
        let events = test_single_event!("\u{1b}[5C");
        assert_matches!(events[0], Ansi::CursorRight(5));
    }

    #[test]
    pub fn test_cursor_left() {
        let events = test_single_event!("\u{1b}[5D");
        assert_matches!(events[0], Ansi::CursorLeft(5));
    }

    #[test]
    pub fn test_reset_all_modes() {
        let events = test_single_event!("\u{1b}[0m");
        assert_matches!(events[0], Ansi::ResetAllModes);
    }

    #[test]
    pub fn test_bold() {
        let events = test_single_event!("\u{1b}[1m");
        assert_matches!(events[0], Ansi::Bold);

        let events = test_single_event!("\u{1b}[22m");
        assert_matches!(events[0], Ansi::ResetBold);
    }

    #[test]
    pub fn test_default_colors() {
        let events = test_single_event!("\u{1b}[39m");
        assert_matches!(events[0], Ansi::DefaultForeground);

        let events = test_single_event!("\u{1b}[49m");
        assert_matches!(events[0], Ansi::DefaultBackground);
    }

    #[test]
    pub fn test_color_by_id() {
        let events = test_single_event!("\u{1b}[38;5;177m");
        if let Ansi::ForegroundColor(c) = events[0] {
            assert_eq!(c, Color::rgb(215, 135, 255));
        } else {
            panic!("expected SetForegroundColor");
        }
    }

    #[test]
    pub fn test_rgb_color() {
        let events = test_single_event!("\u{1b}[38;2;255;0;50m");
        if let Ansi::ForegroundColor(c) = events[0] {
            assert_eq!(c, Color::rgb(255, 0, 50));
        } else {
            panic!("expected SetForegroundColor");
        }
    }

    #[test]
    pub fn test_rgb_color_value_too_large() {
        test_single_invalid_sequence!("\u{1b}[38;2;500;0;50m", "\u{1b}[38;2;500;0;50m");
    }

    #[test]
    pub fn test_rgb_color_value_too_many_args() {
        test_single_invalid_sequence!("\u{1b}[38;2;500;0;50;12m", "\u{1b}[38;2;500;0;50;12m");
    }

    #[test]
    pub fn test_rgb_color_value_too_few_args() {
        test_single_invalid_sequence!("\u{1b}[38;2;500;0m", "\u{1b}[38;2;500;0m");
    }
}
