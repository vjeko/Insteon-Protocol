use std;
use grpc;

use futures::sync::mpsc;
use futures::Sink;
use futures::Future;

use messages_grpc::*;
use messages::*;

#[derive(Debug, Clone)]
pub struct VinsteonRpcImpl {
    pub send: mpsc::Sender<(std::vec::Vec<u8>, u32)>
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