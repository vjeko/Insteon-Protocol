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
    pub send: mpsc::Sender<(std::vec::Vec<u8>, u32)>
}

fn log_result<T, E : Debug>(result: Result<T, E>) -> Result<(()), (())>{
    match result {
        Ok(_) => trace!("Sink flushed"),
        Err(ref e) => trace!("Sink failed! {:?}", e),
    }

    result.map(move |_| (())).map_err(move |_| (()))
}

impl VinsteonRPC for VinsteonRpcImpl {
    fn send_cmd(&self, _m: grpc::RequestOptions, req: CmdMsg) -> grpc::SingleResponse<Ack> {
        let response = Ack::new();

        match req.cmd {
            Some(CmdMsg_oneof_cmd::lightControl(light_control)) =>
                self.send.clone()
                    .send((light_control.device, light_control.level))
                    .then(log_result).wait().unwrap(),

            _ => error!("Unknown command"),
        }

        grpc::SingleResponse::completed(response)
    }
}