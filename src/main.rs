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

use robots::actors::{Actor, ActorSystem, ActorCell, ActorContext, Props};
use serde::{Serialize, Serializer};
use bincode::{serialize, Infinite};
use bus::Bus;

use std::str;
use std::any::Any;
use std::thread;
use std::time::Duration;
use std::sync::Mutex;
use std::sync::Arc;
use std::collections::HashMap;
use std::borrow::Borrow;
use std::ops::Deref;

use log::{LogRecord, LogLevelFilter};
use env_logger::LogBuilder;

use tokio_core::reactor::Core;
use tokio_core::reactor::Remote;

use tokio_io::AsyncRead;
use tokio_serial::*;

use futures::stream::Stream;
use futures::Sink;
use futures::sync::mpsc;
use futures::sync::mpsc::Sender;
use futures::sync::mpsc::Receiver;
use futures::sync::mpsc::UnboundedReceiver;
use futures::sync::mpsc::UnboundedSender;
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

use futures::stream::SplitSink;

struct SerialWriter {
    writer_arc : Arc<Mutex<SplitSink<tokio_io::codec::Framed<tokio_serial::Serial, LineCodec>>>>,
    remote : Remote,
}

impl Actor for SerialWriter {
    fn receive(&self, message: Box<Any>, context: ActorCell) {
        if let Ok(message) = Box::<Any>::downcast::<ActorMsg>(message) {
            match *message {
                ActorMsg::Level((device, level)) => {

                    let shared_writer = self.writer_arc.clone();

                    let scale = level as f64 / 100.0;
                    let brightness = (scale * 255.0) as u8;
                    trace!("Brightness set to {}", brightness);

                    let msg = InsteonMsg::SendStandardMsg{
                        addr_to : device,
                        msg_flags : 15,
                        cmd1 : u8Command(Command::On),
                        cmd2 : brightness
                    };

                    debug!("Sending command: {:?}", msg);

                    let mut struct_repr = serialize(&msg, Infinite).unwrap();
                    struct_repr.drain(..DISCRIMINANT_SIZE);

                    let encoded_msg = [vec![MSG_BEGIN, SEND_STANDARD_MSG], struct_repr].concat();
                    trace!("Encoded command: {:?}", encoded_msg);

                    let mut exclusive_writer = shared_writer.lock().unwrap();
                    exclusive_writer.start_send(encoded_msg).unwrap();
                    exclusive_writer.poll_complete().unwrap();
                }
                
                ActorMsg::Ping => {
                    info!("Received a ping...")
                }
            }
        }
    }
}

impl SerialWriter {
    fn new(tuple : (
        Arc<Mutex<SplitSink<tokio_io::codec::Framed<tokio_serial::Serial, LineCodec>>>>,
        Remote
    )) -> SerialWriter {

        let (writer_arc, remote) = tuple;
        SerialWriter{
            writer_arc : writer_arc,
            remote : remote,
        }
    }
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
    let msg_bus_arc = Arc::new(Mutex::new(Bus::new(10)));

    let actor_system = ActorSystem::new("test".to_owned());

    let props = Props::new(
        Arc::new(SerialWriter::new),
        (writer_arc, core.remote()));
    let serial_writer = actor_system.actor_of(props, "serial_writer".to_owned());

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
