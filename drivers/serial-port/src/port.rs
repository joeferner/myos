use crate::{error::WouldBlockError, retry_until_ok, LineStsFlags};

pub struct SerialPort(u16);

impl SerialPort {
    /// Base port.
    fn port_base(&self) -> u16 {
        self.0
    }

    /// Data port.
    ///
    /// Read and write.
    fn port_data(&self) -> u16 {
        self.port_base()
    }

    #[allow(dead_code)]
    /// Interrupt enable port.
    ///
    /// Write only.
    fn port_int_en(&self) -> u16 {
        self.port_base() + 1
    }

    #[allow(dead_code)]
    /// Fifo control port.
    ///
    /// Write only.
    fn port_fifo_ctrl(&self) -> u16 {
        self.port_base() + 2
    }

    #[allow(dead_code)]
    /// Line control port.
    ///
    /// Write only.
    fn port_line_ctrl(&self) -> u16 {
        self.port_base() + 3
    }

    #[allow(dead_code)]
    /// Modem control port.
    ///
    /// Write only.
    fn port_modem_ctrl(&self) -> u16 {
        self.port_base() + 4
    }

    /// Line status port.
    ///
    /// Read only.
    fn port_line_sts(&self) -> u16 {
        self.port_base() + 5
    }

    /// Creates a new serial port interface on the given I/O base port.
    ///
    /// This function is unsafe because the caller must ensure that the given base address
    /// really points to a serial port device and that the caller has the necessary rights
    /// to perform the I/O operation.
    pub const unsafe fn new(base: u16) -> Self {
        Self(base)
    }

    /// Sends a byte on the serial port.
    pub fn send(&mut self, data: u8) {
        match data {
            8 | 0x7F => {
                self.send_raw(8);
                self.send_raw(b' ');
                self.send_raw(8);
            }
            data => {
                self.send_raw(data);
            }
        }
    }

    /// Sends a raw byte on the serial port, intended for binary data.
    pub fn send_raw(&mut self, data: u8) {
        retry_until_ok!(self.try_send_raw(data))
    }

    /// Tries to send a raw byte on the serial port, intended for binary data.
    pub fn try_send_raw(&mut self, data: u8) -> Result<(), WouldBlockError> {
        if self.line_sts().contains(LineStsFlags::OUTPUT_EMPTY) {
            unsafe {
                x86::io::outb(self.port_data(), data);
            }
            Ok(())
        } else {
            Err(WouldBlockError)
        }
    }

    fn line_sts(&mut self) -> LineStsFlags {
        unsafe { LineStsFlags::from_bits_truncate(x86::io::inb(self.port_line_sts())) }
    }
}

impl<'a> core::fmt::Write for SerialPort {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for byte in s.bytes() {
            self.send(byte);
        }
        Ok(())
    }
}
