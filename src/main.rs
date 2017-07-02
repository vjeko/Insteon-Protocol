#![feature(advanced_slice_patterns, slice_patterns)]
#![feature(discriminant_value, core_intrinsics)]
#![feature(plugin)]
#![plugin(phf_macros)]

extern crate phf;

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
extern crate tls_api;

#[macro_use]
extern crate serde_derive;
extern crate bincode;

use bincode::{serialize, Infinite};

use std::mem;
use std::{io, str};
use std::thread;
use std::time::Duration;
use std::sync::Mutex;
use std::sync::Arc;
use std::intrinsics::discriminant_value;

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
        _dst.extend_from_slice(&_item);
        Ok(())
    }
}


fn main() {

    let mut core = Core::new().unwrap();
    let handle = core.handle();
    const DEFAULT_TTY_PATH: &str = "/dev/ttyUSB0";
    const CHANNEL_BUFFER_SIZE : usize = 10;

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

    let writer_arc = Arc::new(Mutex::new(writer));
    let remote = core.remote();
    let (tx, rx) = mpsc::channel(CHANNEL_BUFFER_SIZE);

    #[derive(Debug, Clone)]
    struct VinsteonRpcImpl {
        send: mpsc::Sender<(std::vec::Vec<u8>, u32)>
    }

    impl VinsteonRPC for VinsteonRpcImpl {
        fn send_cmd(&self, _m: grpc::RequestOptions, req: CmdMsg) -> grpc::SingleResponse<Ack> {
            let response = Ack::new();

            match req.cmd {
                Some(CmdMsg_oneof_cmd::lightControl(light_control)) => {

                    self.send.clone()
                        .send((light_control.device, light_control.level))
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
                (device, level) => {
                    remote.spawn(move |_| {
                        let mut exclusive_writer = shared_writer.lock().unwrap();

                        let scale = level as f64 / 100.0;
                        let brightness = (scale * 255.0) as u8;
                        println!("Brightness set to {}", brightness);


                        let msg = InsteonMsg::SendStandardMsg{
                            addr_to : [device[0], device[1], device[2]],
                            msg_flags : 15,
                            cmd1 : 17,
                            cmd2 : brightness
                        };

                        let mut struct_repr = serialize(&msg, Infinite).unwrap();
                        println!("{:?}", struct_repr);
                        struct_repr.drain(..4);
                        println!("{:?}", struct_repr);

                        let on = [vec![0x02, 0x62], struct_repr].concat();
                        println!("{:?}", on);

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

    let printer = reader.for_each(|s| {
        println!("CMD: {:?}", s);
        Ok(())
    });

    core.run(printer).unwrap();
}
