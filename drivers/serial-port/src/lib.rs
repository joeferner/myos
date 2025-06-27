#![no_std]

// see uart_16550

use conquer_once::{spin::OnceCell, TryInitError};
use spin::Mutex;
use core::fmt::Write;

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

#[doc(hidden)]
pub fn _print(args: ::core::fmt::Arguments) {
    use core::fmt::Write;
    if let Ok(serial1) = SERIAL1.try_get() {
        serial1
            .lock()
            .write_fmt(args)
            .expect("Printing to serial failed");
    }
}

pub fn serial_write_str(s: &str) {
    if let Ok(serial1) = SERIAL1.try_get() {
        serial1
            .lock()
            .write_str(s)
            .expect("Printing to serial failed");
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
