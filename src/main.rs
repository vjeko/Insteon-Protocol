#![feature(advanced_slice_patterns, slice_patterns)]

pub mod insteon_structs;
pub mod messages_grpc;
pub mod messages;

#[macro_use] extern crate log;
extern crate tokio_serial;
extern crate tokio_core;
extern crate tokio_io;
extern crate bytes;
extern crate protobuf;
extern crate grpc;
extern crate futures;
extern crate futures_cpupool;

use std::{io, str};
use std::thread;
use std::time::Duration;
use std::sync::Mutex;
use std::sync::Arc;

use tokio_core::reactor::Core;
use tokio_io::codec::{Decoder, Encoder};
use tokio_io::AsyncRead;
use tokio_serial::*;

use bytes::BytesMut;

use futures::stream::Stream;
use futures::Sink;
use futures::sync::mpsc;
use futures::Future;
use futures_cpupool::CpuPool;

use insteon_structs::*;
use messages_grpc::*;
use messages::*;

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
        Ok(())
    }
}

fn main() {

    let mut core = Core::new().unwrap();
    let handle = core.handle();
    const DEFAULT_TTY_PATH: &str = "/dev/ttyUSB0";


    let tty_path = DEFAULT_TTY_PATH;
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

    let (writer, reader) = port.framed(LineCodec).split();

    let printer = reader.for_each(|s| {
        println!("CMD: {:?}", s);
        Ok(())
    });

    let writer_arc = Arc::new(Mutex::new(writer));
    let remote = core.remote();
    let (tx, rx) = mpsc::channel(10);

    #[derive(Debug, Clone)]
    struct VinsteonRpcImpl {
        send: mpsc::Sender<u32>
    };

    impl VinsteonRPC for VinsteonRpcImpl {
        fn send_cmd(&self, _m: grpc::RequestOptions, req: CmdMsg) -> grpc::SingleResponse<Ack> {
            let response = Ack::new();

            match req.cmd {
                Some(CmdMsg_oneof_cmd::lightControl(light_control)) => {

                    self.send.clone()
                        .send(light_control.level)
                        .then(|tx| {
                            match tx {
                                Ok(_tx) => {
                                    info!("Sink flushed");
                                    Ok(())
                                }
                                Err(e) => {
                                    info!("Sink failed! {:?}", e);
                                    Err(())
                                }
                            }
                        }).wait().unwrap();


                },
                _ => panic!("Unknown command"),
            }

            grpc::SingleResponse::completed(response)
        }
    }

    thread::spawn(move || {
        let _server = VinsteonRPCServer::new_pool(
            "[::]:50051", Default::default(),
            VinsteonRpcImpl{ send : tx.clone() }, CpuPool::new(2)
        );

        loop {
            thread::park();
        }
    });


    // Create a thread that performs some work.
    thread::spawn(move || {

        let writer_future = rx.for_each(|res| {

            let shared_writer = writer_arc.clone();
            match res {
                level => {
                    remote.spawn(move |_| {
                        let scale = level as f64 / 100.0;
                        let brightness = (scale * 255.0) as u8;
                        println!("Brightness set to {}", brightness);

                        let on = vec![0x02, 0x62, 65, 29, 30, 15, 0x11, brightness];
                        //let on = vec![0x02, 0x62, 0x1A, 0xD0, 0xF4, 15, 0x12, 255];

                        let mut exclusive_writer = shared_writer.lock().expect("Unable to lock output");
                        exclusive_writer.start_send(on).unwrap();
                        exclusive_writer.poll_complete().unwrap();
                        Ok(())
                    });
                }
            }

            Ok(())
        });

        writer_future.wait().unwrap();
    });

    core.run(printer).unwrap();
}
