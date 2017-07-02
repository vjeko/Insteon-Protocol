#![feature(advanced_slice_patterns, slice_patterns, core_intrinsics)]
#![feature(plugin)]
#![plugin(phf_macros)]

mod insteon_structs;
mod rpc;
mod codec;
mod messages_grpc;
mod messages;

#[macro_use] extern crate log;
extern crate tokio_serial;
extern crate tokio_core;
extern crate tokio_io;
extern crate bytes;
extern crate protobuf;
extern crate phf;
extern crate grpc;
extern crate futures;
extern crate futures_cpupool;
extern crate tls_api;
extern crate env_logger;

#[macro_use]
extern crate serde_derive;
extern crate bincode;

use bincode::{serialize, Infinite};

use std::str;
use std::thread;
use std::time::Duration;
use std::sync::Mutex;
use std::sync::Arc;

use log::{LogRecord, LogLevelFilter};
use env_logger::LogBuilder;

use tokio_core::reactor::Core;

use tokio_io::AsyncRead;
use tokio_serial::*;

use futures::stream::Stream;
use futures::Sink;
use futures::sync::mpsc;
use futures::Future;
use futures_cpupool::CpuPool;

use insteon_structs::*;
use rpc::VinsteonRpcImpl;
use messages_grpc::*;
use codec::*;

fn setup_logging() {
    let format = |record: &LogRecord| {
        format!("{} - {}", record.level(), record.args())
    };
    let mut builder = LogBuilder::new();
    builder.format(format).filter(Some("vinsteon"), LogLevelFilter::Trace);
    builder.init().unwrap();
}

fn setup_serial_port(core: &Core) -> tokio_io::codec::Framed<tokio_serial::Serial, LineCodec> {
    let handle = core.handle();
    const DEFAULT_TTY_PATH: &str = "/dev/ttyUSB0";

    let settings = SerialPortSettings {
        baud_rate: BaudRate::Baud19200,
        data_bits: DataBits::Eight,
        flow_control: FlowControl::None,
        parity: Parity::None,
        stop_bits: StopBits::One,
        timeout: Duration::from_millis(1),
    };

    info!("Connecting to the serial port...");
    let mut port = tokio_serial::Serial::from_path(DEFAULT_TTY_PATH, &settings, &handle).unwrap();
    port.set_exclusive(false).expect("Unable to set serial port exclusive");
    info!("... done.");

    port.framed(LineCodec)
}

fn main() {

    const CHANNEL_BUFFER_SIZE : usize = 10;
    const GRPC_THREAD_NUM     : usize = 2;
    static GRPC_ADDRESS       : &'static str = "[::]:50051";

    setup_logging();

    let mut core = Core::new().unwrap();
    let serial = setup_serial_port(&core);
    let (writer, reader) = serial.split();

    let writer_arc = Arc::new(Mutex::new(writer));
    let remote = core.remote();

    let (tx, rx) = mpsc::channel(CHANNEL_BUFFER_SIZE);

    info!("Spawning GRPC event handler.");
    let _server = VinsteonRPCServer::new_pool(
        GRPC_ADDRESS, Default::default(),
        VinsteonRpcImpl{ send : tx.clone() }, CpuPool::new(GRPC_THREAD_NUM));

    info!("Spawning serial port writer thread.");
    thread::spawn(move || {

        let writer_future = rx.for_each(|(device, level)| {
            let shared_writer = writer_arc.clone();

            remote.spawn(move |_| {

                let scale = level as f64 / 100.0;
                let brightness = (scale * 255.0) as u8;
                trace!("Brightness set to {}", brightness);

                let msg = InsteonMsg::SendStandardMsg{
                    addr_to : [device[0], device[1], device[2]],
                    msg_flags : 15,
                    cmd1 : 17,
                    cmd2 : brightness
                };

                debug!("Sending command: {:?}", msg);

                let mut struct_repr = serialize(&msg, Infinite).unwrap();
                struct_repr.drain(..4);

                let encoded_msg = [vec![0x02, 0x62], struct_repr].concat();
                trace!("Encoded command: {:?}", encoded_msg);

                let mut exclusive_writer = shared_writer.lock().unwrap();
                exclusive_writer.start_send(encoded_msg).unwrap();
                exclusive_writer.poll_complete().unwrap();
                Ok(())
            });

            Ok(())
        });

        writer_future.wait().unwrap();
    });

    let printer = reader.for_each(|s| {
        info!("Received command: {:?}", s);
        Ok(())
    });

    debug!("Spawning serial port reader thread.");
    core.run(printer).unwrap();
}
