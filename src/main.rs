#![feature(advanced_slice_patterns, slice_patterns)]
#![feature(mpsc_select)]

pub mod insteon_structs;

extern crate futures;
extern crate tokio_serial;
extern crate tokio_core;
extern crate tokio_io;
extern crate bytes;

use std::{io, str};
use std::{thread, time};
use std::sync::mpsc;
use std::time::Duration;
use std::sync::Mutex;
use std::sync::Arc;

use tokio_core::reactor::Core;
use tokio_io::codec::{Decoder, Encoder};
use tokio_io::AsyncRead;
use tokio_serial::*;

use bytes::BytesMut;

use futures::stream::Stream;
use futures::stream::SplitSink;
use futures::Sink;
use futures::sink::Send;
use futures::Future;

use insteon_structs::*;

pub struct LineCodec;

impl Decoder for LineCodec {
    type Item = InsteonMsg;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        const COMMAND_START :u8 = 0x02;
        match src.iter().position(|x| *x == COMMAND_START ) {
            Some(idx) => src.split_to(idx + 1),
            None => {
                src.clear();
                return Ok(None);
            }
        };

        match InsteonMsg::new(&src) {
            Some((msg, msg_size)) => {
                src.split_to(msg_size);
                Ok(Some(msg))
            },
            None => Ok(None)
        }

    }
}

impl Encoder for LineCodec {
    type Item = Vec<u8>;
    type Error = io::Error;

    fn encode(&mut self, _item: Self::Item, _dst: &mut BytesMut) -> Result<(), Self::Error> {
        _dst.extend_from_slice(&_item);
        Ok(())
    }
}


fn main() {

    let mut core = Core::new().unwrap();
    let handle = core.handle();

    let tty_path = "/dev/ttyUSB0";
    let settings = SerialPortSettings {
        baud_rate: BaudRate::Baud19200,
        data_bits: DataBits::Eight,
        flow_control: FlowControl::None,
        parity: Parity::None,
        stop_bits: StopBits::One,
        timeout: Duration::from_millis(1),
    };


    let mut port = tokio_serial::Serial::from_path(tty_path, &settings, &handle).unwrap();
    port.set_exclusive(false).expect("Unable to set serial port exclusive");

    let (mut writer, reader) = port.framed(LineCodec).split();

    let printer = reader.for_each(|s| {
        println!("CMD: {:?}", s);
        Ok(())
    });

    let original = Arc::new(Mutex::new(writer));

    let remote = core.remote();

    // Create a thread that performs some work.
    thread::spawn(move || {
        loop {
            // INSERT WORK HERE - the work should be modeled as having a _future_ result.
            let www1 = original.clone();
            let www2 = original.clone();

            // In this fake example, we do not care about the values of the `Ok` and `Err`
            // variants. thus, we can use `()` for both.
            // Note: `::futures::done()` will be called ::futures::result() in later
            // versions of the future crate.

            let f = ::futures::done::<(), ()>(Ok(()));
            // END WORK

            // `remote.spawn` accepts a closure with a single parameter of type `&Handle`.
            // In this example, the `&Handle` is not needed. The future returned from the
            // closure will be executed.
            //
            // Note: We must use `remote.spawn()` instead of `handle.spawn()` because the
            // Core was created on a different thread.
            std::thread::sleep(time::Duration::from_secs(1));

            remote.spawn(move |_| {
                let mut wr = www1.lock().expect("Unable to lock output");

                //let on = vec![0x02, 0x62, 65, 29, 30, 15, 0x12, 255];
                let on = vec![0x02, 0x62, 0x1A, 0xD0, 0xF4, 15, 0x12, 255];
                wr.start_send(on);
                wr.poll_complete();
                Ok(())
            });

            std::thread::sleep(time::Duration::from_secs(1));

            remote.spawn(move |_| {
                let mut wr = www2.lock().expect("Unable to lock output");

                //let off = vec![0x02, 0x62, 65, 29, 30, 15, 0x14, 0];
                let off = vec![0x02, 0x62, 0x1A, 0xD0, 0xF4, 15, 0x14, 0];

                wr.start_send(off);
                wr.poll_complete();
                Ok(())
            });
        }
    });

    core.run(printer).unwrap();
}
