#![feature(advanced_slice_patterns, slice_patterns)]
extern crate serial;

use std::time::Duration;
use std::vec::Vec;

use std::io::prelude::*;
use serial::prelude::*;

const SETTINGS: serial::PortSettings = serial::PortSettings {
    baud_rate:    serial::Baud19200,
    char_size:    serial::Bits8,
    parity:       serial::ParityNone,
    stop_bits:    serial::Stop1,
    flow_control: serial::FlowNone
};

struct Port {
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

fn print_cmd(port: &mut Port) -> Vec<u8> {
    const COMMAND_START :u8 = 0x02;

    const STANDARD_MSG :u8 = 0x50;
    const EXTENDED_MSG :u8 = 0x51;
    const X10_RECEIVED :u8 = 0x52;
    const ALL_LINKING_COMPLETED :u8 = 0x53;
    const BUTTON_EVENT_REPORT :u8 = 0x54;
    const USER_RESET_DETECTED :u8 = 0x55;

    port.skip_while(|x| x != &COMMAND_START).next();

    let command_type :u8 = port.take(1).next().unwrap();
    println!("commandType {:02X}", &command_type);

    match command_type {
        STANDARD_MSG => port.take(9).collect::<Vec<_>>(),
        EXTENDED_MSG => port.take(23).collect::<Vec<_>>(),
        X10_RECEIVED => port.take(23).collect::<Vec<_>>(),
        ALL_LINKING_COMPLETED => port.take(8).collect::<Vec<_>>(),
        BUTTON_EVENT_REPORT => port.take(1).collect::<Vec<_>>(),
        USER_RESET_DETECTED => vec![],
        _ => vec![],
    }

}

pub fn to_hex_string(bytes: Vec<u8>) -> String {
    let strings: Vec<String> = bytes.iter()
        .map(|b| format!("{:02X}", b))
        .collect();
    strings.join(" ")
}


fn main() {
    let port_path = String::from("/dev/ttyUSB0");
    let mut port = Port::new(port_path);

    loop {
        let cmd = print_cmd(&mut port);
        println!("cmd {}", to_hex_string(cmd));
    }
}