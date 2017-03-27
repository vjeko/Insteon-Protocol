#![feature(advanced_slice_patterns, slice_patterns)]
extern crate serial;

use std::env;
use std::time::Duration;
use std::vec::Vec;
use std::iter;

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
        (self.port_impl.configure(&SETTINGS));
        (self.port_impl.set_timeout(Duration::from_millis(1000)));
    }
}

// Implement `Iterator` for `Fibonacci`.
// The `Iterator` trait only requires a method to be defined for the `next` element.
impl Iterator for Port {
    type Item = u8;

    // Here, we define the sequence using `.curr` and `.next`.
    // The return type is `Option<T>`:
    //     * When the `Iterator` is finished, `None` is returned.
    //     * Otherwise, the next value is wrapped in `Some` and returned.
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
    let commandStart :u8 = 2;
    port.take_while(|x| x != &commandStart).collect::<Vec<_>>()
}

pub fn to_hex_string(bytes: Vec<u8>) -> String {
    let strs: Vec<String> = bytes.iter()
        .map(|b| format!("{:02X}", b))
        .collect();
    strs.connect(" ")
}


fn main() {

    let port_path = String::from("/dev/ttyUSB0");
    let mut port = Port::new(port_path);

    loop {
        let cmd = print_cmd(&mut port);
        let RCV_STD_MSG : u8 = 0x50;

        match cmd.as_slice() {
            &[RCV_STD_MSG,
                from_high, from_mid, from_low,
                to_high, to_mid, to_low,
                msg_flags, msg_1, msg_2
            ] => {
                println!("INSTEON Standard Message Received (0x50)");
                println!("From:        {}", to_hex_string(vec![from_high, from_mid, from_low]));
                println!("To:          {}", to_hex_string(vec![to_high, to_mid, to_low]));
                println!("Msg Flags:   {}", to_hex_string(vec![msg_flags]));
                println!("Msg:         {}", to_hex_string(vec![msg_1, msg_2]));

            },
            _   => println!("Not implemented")
        }

        //println!("{:?}", to_hex_string(cmd))
    }
}