#![no_std]

// see uart_16550

use conquer_once::{spin::OnceCell, TryInitError};
use spin::Mutex;

pub const SERIAL1_ADDR: u16 = 0x03f8;

static SERIAL1: OnceCell<Mutex<SerialPort>> = OnceCell::uninit();

pub unsafe fn serial1_init() -> Result<(), TryInitError> {
    SERIAL1.try_init_once(|| {
        let serial_port = unsafe { SerialPort::new(SERIAL1_ADDR) };
        spin::Mutex::new(serial_port)
    })
}

pub struct SerialPort {
    inner: uart_16550::SerialPort,
}

impl SerialPort {
    pub unsafe fn new(addr: u16) -> Self {
        let mut inner = unsafe { uart_16550::SerialPort::new(addr) };
        inner.init();
        Self { inner }
    }
}

impl<'a> core::fmt::Write for SerialPort {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.inner.write_str(s)
    }
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
