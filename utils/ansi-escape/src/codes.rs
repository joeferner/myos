use core::fmt::{self, Display};

use crate::Color;

pub struct WithForeground<'a, T: Display + ?Sized> {
    color: Color,
    value: &'a T,
}

impl<'a, T: Display + ?Sized> Display for WithForeground<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "\u{1b}[38;2;{};{};{}m{}\u{1b}[39m",
            self.color.red, self.color.green, self.color.blue, self.value
        )
    }
}

pub struct WithBackground<'a, T: Display + ?Sized> {
    color: Color,
    value: &'a T,
}

impl<'a, T: Display + ?Sized> Display for WithBackground<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "\u{1b}[48;2;{};{};{}m{}\u{1b}[49m",
            self.color.red, self.color.green, self.color.blue, self.value
        )
    }
}

pub struct WithBold<'a, T: Display + ?Sized> {
    value: &'a T,
}

impl<'a, T: Display + ?Sized> Display for WithBold<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\u{1b}[1m{}\u{1b}[22m", self.value)
    }
}

pub struct MoveCursor {
    line: u8,
    column: u8,
}

impl Display for MoveCursor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\u{1b}[{};{}H", self.line, self.column)
    }
}

pub struct Ansi {}

impl Ansi {
    pub fn fg<'a, T: Display + ?Sized>(color: Color, value: &'a T) -> WithForeground<'a, T> {
        WithForeground { color, value }
    }

    pub fn bg<'a, T: Display + ?Sized>(color: Color, value: &'a T) -> WithBackground<'a, T> {
        WithBackground { color, value }
    }

    pub fn bold<'a, T: Display + ?Sized>(value: &'a T) -> WithBold<'a, T> {
        WithBold { value }
    }

    pub fn move_cursor(line: u8, column: u8) -> MoveCursor {
        MoveCursor { line, column }
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use super::*;
    use std::format;

    #[test]
    pub fn bold() {
        assert_eq!(
            format!("{}", Ansi::bold("Hello, world!")),
            "\u{1b}[1mHello, world!\u{1b}[22m"
        );
    }

    #[test]
    pub fn move_cursor() {
        assert_eq!(format!("{}", Ansi::move_cursor(10, 12)), "\u{1b}[10;12H");
    }

    #[test]
    pub fn foreground_color() {
        let color = Color {
            red: 255,
            green: 0,
            blue: 0,
        };
        assert_eq!(
            format!("{}", Ansi::fg(color, "Hello, world!")),
            "\u{1b}[38;2;255;0;0mHello, world!\u{1b}[39m"
        );
    }

    #[test]
    pub fn background_color() {
        let color = Color {
            red: 255,
            green: 0,
            blue: 0,
        };
        assert_eq!(
            format!("{}", Ansi::bg(color, "Hello, world!")),
            "\u{1b}[48;2;255;0;0mHello, world!\u{1b}[49m"
        );
    }

    #[test]
    pub fn foreground_and_background_color() {
        let color = Color {
            red: 255,
            green: 0,
            blue: 0,
        };
        assert_eq!(
            format!("{}", Ansi::fg(color, &Ansi::bg(color, "Hello, world!"))),
            "\u{1b}[38;2;255;0;0m\u{1b}[48;2;255;0;0mHello, world!\u{1b}[49m\u{1b}[39m"
        );
    }
}
