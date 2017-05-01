#![feature(advanced_slice_patterns, slice_patterns)]

pub mod insteon_structs;
pub mod tty;

use insteon_structs::*;
use tty::*;

fn print_cmd(port: &mut Port) -> InsteonMsg {
    const COMMAND_START :u8 = 0x02;
    port.skip_while(|x| x != &COMMAND_START).next();
    return InsteonMsg::new(port)
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
        let msg = print_cmd(&mut port);
        println!("msg {:?}", msg);
    }
}