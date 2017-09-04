#![feature(advanced_slice_patterns, slice_patterns)]
#![feature(plugin)]
#![plugin(phf_macros)]

mod insteon_structs;
mod rpc;
mod codec;
mod messages_grpc;
mod messages;
mod serial_writer;

#[macro_use] extern crate log;
extern crate tokio_serial;
extern crate tokio_core;
extern crate tokio_io;
extern crate bytes;
extern crate protobuf;
extern crate phf;
extern crate grpc;
extern crate bus;
extern crate futures;
extern crate futures_cpupool;
extern crate tls_api;
extern crate env_logger;


#[macro_use] extern crate serde_derive;
extern crate bincode;
extern crate serde;
extern crate serde_json;
extern crate robots;

use robots::actors::{ActorSystem, Props};
use bus::Bus;

use std::str;
use std::thread;
use std::time::Duration;
use std::sync::Mutex;
use std::sync::Arc;
use std::collections::HashMap;

use log::{LogRecord, LogLevelFilter};
use env_logger::LogBuilder;

use tokio_core::reactor::Core;

use tokio_io::AsyncRead;
use tokio_serial::*;

use futures::stream::Stream;
use futures_cpupool::CpuPool;

use rpc::VinsteonRpcImpl;
use messages_grpc::*;
use codec::*;
use serial_writer::SerialWriterActor;
use serial_writer::SerialReaderActor;


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

    const GRPC_THREAD_NUM     : usize = 2;
    static GRPC_ADDRESS       : &'static str = "[::]:50051";

    setup_logging();

    let mut core = Core::new().unwrap();
    let serial = setup_serial_port(&core);
    let (writer, reader) = serial.split();

    let writer_arc = Arc::new(Mutex::new(writer));
    let msg_bus_arc = Arc::new(Mutex::new(Bus::new(10)));

    let actor_system = ActorSystem::new("test".to_owned());

    let writer_props = Props::new(
        Arc::new(SerialWriterActor::new),
        (writer_arc));
    let serial_writer = actor_system.actor_of(writer_props, "serial_writer".to_owned());

    let reader_props = Props::new(
        Arc::new(SerialReaderActor::new), ());

    let printer = reader.for_each(|s| {
        info!("Received command: {:?}", s);
        msg_bus_arc.lock().unwrap().broadcast(s);
        Ok(())
    });

    info!("Spawning GRPC event handler.");
    let _server = VinsteonRPCServer::new_pool(
        GRPC_ADDRESS, Default::default(),
        VinsteonRpcImpl{
            actor : serial_writer.clone(),
            msg_bus : msg_bus_arc.clone() }, CpuPool::new(GRPC_THREAD_NUM));

    info!("Spawning persitence phread.");
    thread::spawn(|| {
        let mut devices : HashMap<String, [u8; 3]> = std::collections::HashMap::new();
        devices.insert("".into(), [1,2,3]);
        println!("{}", serde_json::to_string(&devices).unwrap());
    });

    debug!("Spawning serial port reader thread.");
    core.run(printer).unwrap();
}
