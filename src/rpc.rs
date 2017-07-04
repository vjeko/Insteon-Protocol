use std;
use grpc;

use std::fmt::Debug;

use futures::sync::mpsc;
use futures::Sink;
use futures::Future;

use messages_grpc::*;
use messages::*;

#[derive(Debug, Clone)]
pub struct VinsteonRpcImpl {
    pub send: mpsc::Sender<([u8; 4], u32)>
}

fn log_result<T, E : Debug>(result: Result<T, E>) -> Result<(()), (())>{
    match result {
        Ok(_) => trace!("Sink flushed"),
        Err(ref e) => trace!("Sink failed! {:?}", e),
    }

    result.map(move |_| (())).map_err(move |_| (()))
}

fn u32_u8(x:u32) -> [u8; 4] {
    let b1 : u8 = ((x >> 24) & 0xff) as u8;
    let b2 : u8 = ((x >> 16) & 0xff) as u8;
    let b3 : u8 = ((x >> 8) & 0xff) as u8;
    let b4 : u8 = (x & 0xff) as u8;
    return [b1, b2, b3, b4]
}

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
}