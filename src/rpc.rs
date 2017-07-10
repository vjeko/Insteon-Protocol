use grpc;
use bus::Bus;

use std::time::Duration;
use std::fmt::Debug;
use std::sync::Mutex;
use std::sync::Arc;

use futures::sync::mpsc;
use futures::Sink;
use futures::Future;

use messages_grpc::*;
use messages::*;
use insteon_structs::*;

#[derive(Clone)]
pub struct VinsteonRpcImpl {
    pub send :    mpsc::Sender<([u8; 3], u32)>,
    pub msg_bus : Arc<Mutex<Bus<InsteonMsg>>>
}

fn log_result<T, E : Debug>(result: Result<T, E>) -> Result<(()), (())>{
    match result {
        Ok(_) => trace!("Sink flushed"),
        Err(ref e) => trace!("Sink failed! {:?}", e),
    }

    result.map(move |_| (())).map_err(move |_| (()))
}

fn u32_u8(x:u32) -> [u8; 3] {
    let _b1 : u8 = ((x >> 24) & 0xff) as u8;
    let b2 : u8 = ((x >> 16) & 0xff) as u8;
    let b3 : u8 = ((x >> 8) & 0xff) as u8;
    let b4 : u8 = (x & 0xff) as u8;
    [b2, b3, b4]
}

pub const WAIT_ACK_RETRIES :usize = 3;
pub const SEND_RETRIES :usize = 10;

impl VinsteonRPC for VinsteonRpcImpl {

    fn send_cmd(&self, _m: grpc::RequestOptions, req: CmdMsg) -> grpc::SingleResponse<Ack> {
        let response = Ack::new();

        match req.cmd {
            Some(CmdMsg_oneof_cmd::lightControl(light_control)) =>
                self.send.clone()
                    .send((u32_u8(light_control.device), light_control.level))
                    .then(log_result).wait().unwrap(),

            _ => error!("Unknown command"),
        }

        grpc::SingleResponse::completed(response)
    }

    fn send_cmd_reliable(&self, _m: grpc::RequestOptions, req: CmdMsg) -> grpc::SingleResponse<Ack> {
        let mut bus = self.msg_bus.lock().unwrap().add_rx();

        let mut get_ack = |_from : [u8; 3], _cmd2: u8| -> Result<(), ()> {
            match bus.recv_timeout(Duration::from_millis(200)) {
                Ok(InsteonMsg::StandardMsg { addr_from: _from, cmd2: _cmd2, ..}) => {
                    debug!("Received the acknowledgment from: {:?} ", _from);
                    Ok(())},
                Ok(other) => {
                    trace!("Not matching the pattern: {:?}", other);
                    Err(())},
                Err(_) => {
                    trace!("Timeout");
                    Err(())}
            }
        };

        fn retry(f: &mut FnMut() -> Result<(), ()>, retries : usize) -> Result<(), ()> {
            match (retries, f()) {
                (0, _) => Err(()),
                (_, ok@Ok(())) => ok,
                (_, Err(())) => retry(f, retries - 1)
            }
        };

        let response = Ack::new();

        match req.cmd {
            Some(CmdMsg_oneof_cmd::lightControl(light_control)) => {
                let dst = u32_u8(light_control.device);

                let mut send_closure = || self.send.clone()
                    .send((dst, light_control.level))
                    .then(log_result).wait().unwrap();

                debug!("Waiting for the confirmation");
                let mut get_ack_closure = || get_ack(dst, light_control.level as u8);
                let mut sync_wait_closure = || {
                    send_closure();
                    retry(&mut get_ack_closure, WAIT_ACK_RETRIES)
                };

                retry(&mut sync_wait_closure, SEND_RETRIES);
            },

            _ => error!("Unknown command"),
        }

        grpc::SingleResponse::completed(response)
    }
}