
use std::time::Duration;
use std::thread;

#[macro_use] extern crate log;
extern crate simplelog;
use simplelog::{TermLogger, LevelFilter};

extern crate structopt;
use structopt::StructOpt;

#[macro_use(block)]
extern crate nb;

use embedded_gps::Gps;

#[derive(Debug, StructOpt)]
#[structopt(name = "gps-util")]
pub struct Options {
    #[structopt(short = "p", long = "serial.port", default_value = "/dev/serial0", env = "GPS_PORT")]
    /// Serial port for receiving NMEA data
    pub serial_port: String,

    #[structopt(short = "b", long = "serial.baud", default_value = "115200", env = "GPS_BAUD")]
    /// Client ID for MQTT connection
    pub baud: usize,

    #[structopt(long = "log-level", default_value = "info")]
    /// Enable verbose logging
    pub level: LevelFilter,
}

fn main() {
    // Load options
    let opts = Options::from_args();
    debug!("Options: {:?}", opts);

    // Setup logging
    TermLogger::init(opts.level, simplelog::Config::default()).unwrap();

    // Connect to GPS device
    let mut gps = match Gps::linux(&opts.serial_port, opts.baud) {
        Ok(g) => g,
        Err(e) => {
            error!("Error creating GPS instance: {:?}", e);
            return;
        }
    };

    loop {
        // Poll for GPS updates
        if let Err(e) = block!(gps.poll()) {
                error!("Error polling GPS: {:?}", e);
                return;
        }

        // If there's a lock, print location information
        if let Some((lat, lng, alt)) = gps.state() {
            info!("lat: {:.4}, lng: {:.4}, alt: {:.4}", lat, lng, alt);
        }

        // Sleep for a while
        thread::sleep(Duration::from_millis(10));
    }

}
