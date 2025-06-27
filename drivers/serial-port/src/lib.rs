#![no_std]

// see https://github.com/rust-osdev/uart_16550/blob/master/src/port.rs

use conquer_once::{spin::OnceCell, TryInitError};
use spin::Mutex;
use port::SerialPort;
use bitflags::bitflags;

pub mod error;
pub mod port;

pub const SERIAL1_ADDR: u16 = 0x03f8;

static SERIAL1: OnceCell<Mutex<SerialPort>> = OnceCell::uninit();

pub unsafe fn serial1_init() -> Result<(), TryInitError> {
    SERIAL1.try_init_once(|| {
        let serial_port = unsafe { SerialPort::new(SERIAL1_ADDR) };
        spin::Mutex::new(serial_port)
    })
}

bitflags! {
    /// Line status flags
    #[repr(transparent)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    struct LineStsFlags: u8 {
        const INPUT_FULL = 1;
        // 1 to 4 unknown
        const OUTPUT_EMPTY = 1 << 5;
        // 6 and 7 unknown
    }
}

#[macro_export]
macro_rules! retry_until_ok {
    ($cond:expr) => {
        loop {
            if let Ok(ok) = $cond {
                break ok;
            }
            core::hint::spin_loop();
        }
    };
}

pub fn serial_print_args(args: ::core::fmt::Arguments) -> core::fmt::Result {
    use core::fmt::Write;
    if let Ok(serial1) = SERIAL1.try_get() {
        serial1.lock().write_fmt(args)
    } else {
        Ok(())
    }
}

/// Prints to the host through the serial interface.
#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        $crate::serial::_print(format_args!($($arg)*));
    };
}

/// Prints to the host through the serial interface, appending a newline.
#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($fmt:expr) => ($crate::serial_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::serial_print!(
        concat!($fmt, "\n"), $($arg)*));
}
