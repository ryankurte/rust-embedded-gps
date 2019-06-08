
#![no_std]

use core::marker::PhantomData;
use core::fmt::Debug;
use core::ops::Deref;

extern crate embedded_hal;
use embedded_hal::serial::Read as SerialRead;

#[cfg(feature = "linux")]
pub mod linux;

#[macro_use]
extern crate log;

extern crate nb;

extern crate nmea;
use nmea::{Nmea, FixType};

/// Gps object wraps a serial port and nmea parser to provide
/// a simple interface to handle physical GPS devices
#[derive(Debug)]
pub struct Gps<S, E, T> {
    serial: S,
    index: usize,
    buffer: T,
    nmea: Nmea,
    _e: PhantomData<E>,
}

/// GpsError object wraps possible GPS errors
#[derive(Debug, Clone, PartialEq)]
pub enum GpsError<E> {
    Serial(E),
    BufferOverrun,
    Nmea,
    Utf8(core::str::Utf8Error)
}

/// Deref allows fields on the internal Nmea object to be accessed
impl <S, E, T> Deref for Gps<S, E, T> {
    type Target = Nmea;
    fn deref(&self) -> &Nmea {
        &self.nmea
    }
}


impl <S, E, T> Gps<S, E, T> 
where 
    S: SerialRead<u8, Error=E>,
    E: PartialEq + Debug,
    T: AsRef<[u8]> + AsMut<[u8]> + Debug,
{
    /// New creates a new GPS object using the provided serial implementation and buffr
    pub fn new(serial: S, buffer: T) -> Self {
        Gps{ serial, index: 0, buffer, _e: PhantomData, nmea: Nmea::new() }
    }

    /// Poll polls the underlying serial port and updates the GPS state
    /// This returns WouldBlock until valid information is available, then a boolean to indicate gps lock status
    pub fn poll(&mut self) -> nb::Result<bool, GpsError<E>> {
        use nb::Error::*;

        // Poll for serial data
        let c = self.serial.read().map_err(|e| 
            match e {
                WouldBlock => WouldBlock,
                Other(e) => {
                    error!("GPS serial error: {:?}", e);
                    Other(GpsError::Serial(e))
                },
            }
        )?;

        // Update buffer data
        let buff = self.buffer.as_mut();

        buff[self.index] = c;
        self.index += 1;

        // Check buffer bounds
        if self.index > buff.len() {
            error!("GPS buffer overrun");
            return Err(Other(GpsError::BufferOverrun))
        }

        drop(buff);

        // Continue polling until end char
        if c != '\n' as u8 {
            return Err(WouldBlock)
        }

        // Attempt to parse NMEA string
        let res = self.update();
        
        // Reset buffer for next operation
        self.index = 0;
    
        res
    }

    /// Fetch state
    /// Returns (lat, lng, alt) if the GPS is locked
    pub fn state(&self) -> Option<(f64, f64, f32)> {
        match (self.latitude(), self.longitude(), self.altitude()) {
            (Some(lat), Some(lng), Some(alt)) => Some((lat, lng, alt)),
            _ => None
        }
    }

    fn update(&mut self) -> nb::Result<bool, GpsError<E>> {
        use nb::Error::*;

        let buff = self.buffer.as_ref();

        debug!("Attempting to parse: {:?}", &buff[..self.index]);

        let t = self.nmea.parse_for_fix(&buff[..self.index]).map_err(|_e| Other(GpsError::Nmea) )?;

        return Ok(t != FixType::Invalid)
    }

}
