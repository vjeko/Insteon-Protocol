extern crate serial;

use std::time::Duration;
use std::vec::Vec;
use tty::serial::SerialPort;

use std::io::prelude::*;

pub const SETTINGS: serial::PortSettings = serial::PortSettings {
    baud_rate:    serial::Baud19200,
    char_size:    serial::Bits8,
    parity:       serial::ParityNone,
    stop_bits:    serial::Stop1,
    flow_control: serial::FlowNone
};

pub struct Port {
    port_impl: serial::SystemPort,
    buf: Vec<u8>
}

impl Port {
    pub fn new(location: String) -> Port {
        let mut port = Port {
            port_impl:     serial::open(&location).unwrap(),
            buf:           vec![0; 1]
        };

        port.setup();
        return port
    }

    pub fn send(&mut self, buf: &[u8]) {
        self.port_impl.write(buf);
    }

    pub fn setup(&mut self) {
        match self.port_impl.configure(&SETTINGS) {
            Ok(_) => println!("successfully configured the serial port"),
            Err(err) => panic!("unable to configure the serial port: {:?}", err),
        };

        match self.port_impl.set_timeout(Duration::from_millis(1000)) {
            Ok(_) => println!("successfully changed the timeout value for the serial port"),
            Err(err) => panic!("unable to change the timeout value for the serial port: {:?}", err),
        };
    }
}

// Implement `Iterator` for `Port`.
// The `Iterator` trait only requires a method to be defined for the `next` element.
impl Iterator for Port {
    type Item = u8;

    fn next(&mut self) -> Option<u8> {
        loop {
            match self.port_impl.read(&mut self.buf[..]) {
                Ok(_) => return Some(self.buf[0]),
                Err(_) => continue
            }
        }
    }
}