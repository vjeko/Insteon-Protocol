extern crate tokio_serial;
extern crate tokio_io;

use robots::actors::{Actor, ActorCell};
use bincode::{serialize, Infinite};

use std::any::Any;
use std::sync::Mutex;
use std::sync::Arc;

use futures::Sink;
use futures::stream::SplitSink;

use insteon_structs::*;
use codec::*;

pub struct SerialWriterActor {
    writer_arc : Arc<Mutex<SplitSink<tokio_io::codec::Framed<tokio_serial::Serial, LineCodec>>>>,
}

impl Actor for SerialWriterActor {
    fn receive(&self, message: Box<Any>, _context: ActorCell) {
        if let Ok(message) = Box::<Any>::downcast::<ActorMsg>(message) {
            match *message {
                ActorMsg::Level((device, level)) => {

                    let shared_writer = self.writer_arc.clone();

                    let scale = level as f64 / 100.0;
                    let brightness = (scale * 255.0) as u8;
                    trace!("Brightness set to {}", brightness);

                    let msg = InsteonMsg::SendStandardMsg{
                        addr_to : device,
                        msg_flags : Flags::DIRECT_MSG | Flags::STANDARD_MSG |
                                    Flags::MSG_REMAINING_3 | Flags::RETRANSMIT_3,
                        cmd1 : u8_command(Command::On),
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
            }
        }
    }
}

impl SerialWriterActor {
    pub fn new(writer_arc :
        Arc<Mutex<SplitSink<tokio_io::codec::Framed<tokio_serial::Serial, LineCodec>>>>,
    ) -> SerialWriterActor {

        SerialWriterActor{
            writer_arc : writer_arc,
        }
    }
}