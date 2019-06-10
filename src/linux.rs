
extern crate std;
use std::vec::Vec;
use std::io::{Error, ErrorKind};

extern crate linux_embedded_hal;
use linux_embedded_hal::Serial;
use linux_embedded_hal::serial_unix::{TTYPort};
use linux_embedded_hal::serial_core::{SerialDevice, SerialPortSettings, BaudRate};

use crate::Gps;

impl Gps<Serial, ErrorKind, Vec<u8>> {

    /// Constructor for a linux GPS object
    /// This uses serial_unix and allocates a vector internally
    pub fn linux(port: &str, baud: usize) -> Result<Self, Error> {
        let p = std::path::Path::new(port);

        let mut p = TTYPort::open(p)?;
        let mut settings = p.read_settings()?;
        settings.set_baud_rate(BaudRate::from_speed(baud))?;
        p.write_settings(&settings)?;

        let p = Serial(p);
        let b = std::vec![0u8; 128];

        Ok(Gps::new(p, b))
    }
}
