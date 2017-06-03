#![feature(advanced_slice_patterns, slice_patterns)]
#![feature(mpsc_select)]

pub mod insteon_structs;
pub mod tty;

use insteon_structs::*;
use tty::*;
use std::{thread, time};
use std::sync::mpsc;

fn receive_cmd(port: &mut Port) -> InsteonMsg {
    const COMMAND_START :u8 = 0x02;
    port.skip_while(|x| x != &COMMAND_START).next();
    return InsteonMsg::new(port)
}

fn main() {
    let port_path = String::from("/dev/ttyUSB0");
    let mut port = Port::new(port_path);

    let (tx_main, rx_main) = mpsc::channel();

    thread::spawn(move || {
        loop {
            let msg = receive_cmd(&mut port);
            tx_main.send(msg).unwrap();
        }
    });

    thread::spawn(move || {
        loop {
            std::thread::sleep(time::Duration::from_secs(2));
            let on = vec![0x02, 0x62, 65, 29, 30, 64, 17, 1];
            //port.send(&on);
            std::thread::sleep(time::Duration::from_secs(2));
            let off = vec![0x02, 0x62, 65, 29, 30, 64, 19, 1];;
            //port.send(&off);

            println!("==================");
        }
    });

    thread::spawn(move || {
        loop {
            println!("msg {:?}", rx_main.recv().unwrap());
        }
    }).join().unwrap();
}