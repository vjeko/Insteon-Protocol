use grpc;
use bus::Bus;

use std::thread;
use std::time::Duration;
use std::fmt::Debug;
use std::sync::Mutex;
use std::sync::Arc;
use futures::Future;

use tokio_timer::Timer;
use tokio_core::reactor::{Remote};
use robots::actors::{ActorContext, ActorRef, ActorSystem, Actor, Any, ActorCell, Props};

use messages_grpc::*;
use messages::*;
use insteon_structs::*;

#[derive(Clone)]
pub enum RpcActorMsg {
    Set(LightControl),
    SetReliable(CmdMsg),
}

#[derive(Clone)]
pub enum RpcReqActorMsg {
    Set(ActorRef, LightControl),
    SetReliable(ActorRef, CmdMsg),
    Timeout(usize)
}

pub struct RpcReqActor {
    pub ser_tx_actor : ActorRef,
    pub msg_bus      : Arc<Mutex<Bus<InsteonMsg>>>,
    pub req          : Mutex<Option<(ActorRef, CmdMsg)>>,
    pub event_loop   : Remote,
}

unsafe impl Sync for RpcReqActor { }

impl RpcReqActor {
    pub fn new(tuple : (ActorRef, Arc<Mutex<Bus<InsteonMsg>>>, Remote)) -> RpcReqActor {
        let (ser_tx_actor, msg_bus, event_loop) = tuple;
        RpcReqActor {
            ser_tx_actor : ser_tx_actor,
            msg_bus : msg_bus,
            req : Mutex::new(None),
            event_loop: event_loop,
        }
    }

    pub fn handle_insteon_msg(&self, message: InsteonMsg, context: ActorCell) {
        info!("RpcReqActor: Received InsteonMsg: {:?}", message);
        if let Some((ref future,
                     CmdMsg{ cmd : Some(CmdMsg_oneof_cmd::lightControl(ref lc)), .. })
        ) = *self.req.lock().unwrap() {
            let _device_addr = u32_u8(lc.device);
            match message {
                InsteonMsg::StandardMsg {addr_from : _device_addr, ..} => {
                    info!("Received the ACK: {:?}", message);
                    context.complete(future.clone(), Ack::new());
                    info!("Killing myself...");
                    context.kill_me();
                },
                _ => (),
            }
        }
    }

    pub fn handle_rpc_msg(&self, message: RpcReqActorMsg, context: ActorCell) {
        match message {
            RpcReqActorMsg::Set(ref future, ref light_control) => {
                info!("RpcReqActor received RpcReqActorMsg::Set");
                self.ser_tx_actor.tell_to(self.ser_tx_actor.clone(),
                    ActorMsg::Level((u32_u8(light_control.device), light_control.level)));
                context.complete(future.clone(), Ack::new());
            },
            RpcReqActorMsg::SetReliable(ref future, ref cmd) => {
                info!("RpcReqActor received RpcReqActorMsg::SetReliable");
                self.send_cmd_once(cmd.clone());

                let mut interior = self.req.lock().unwrap();
                *interior = Some((future.clone(), cmd.clone()));;

                let actor_ref = context.actor_ref().clone();
                thread::spawn(move ||{
                    Timer::default().sleep(Duration::from_secs(1))
                        .wait().expect("Unable to sleep.");;
                    actor_ref.tell_to(actor_ref.clone(), RpcReqActorMsg::Timeout(0));
                });
            },
            RpcReqActorMsg::Timeout(SEND_RETRIES) => {
                info!("Reached the maximum number of retries, giving up...");
                info!("Killing myself...", );
                context.kill_me();
            },

            RpcReqActorMsg::Timeout(retries) => {
                info!("Retrying...");
                if let Some((_, ref cmd)) = *self.req.lock().unwrap() {
                    self.send_cmd_once(cmd.clone());
                    let actor_ref = context.actor_ref().clone();
                    thread::spawn(move ||{
                        Timer::default().sleep(Duration::from_secs(ACK_WAIT_INTERVAL_SEC))
                            .wait().expect("Unable to sleep.");
                        actor_ref.tell_to(actor_ref.clone(), RpcReqActorMsg::Timeout(retries + 1));
                    });
                }
            },
        }
    }

    fn send_cmd_once(&self, req: CmdMsg) -> Ack {
        match req.cmd {
            Some(CmdMsg_oneof_cmd::lightControl(light_control)) => {
                let dst = u32_u8(light_control.device);
                self.ser_tx_actor.tell_to(self.ser_tx_actor.clone(),
                                          ActorMsg::Level((dst, light_control.level)));
            },

            _ => error!("Unknown command"),
        };

        Ack::new()
    }
}

impl Actor for RpcReqActor {

    fn receive(&self, msg: Box<Any>, context: ActorCell) {
        match msg.downcast_ref::<RpcReqActorMsg>() {
            Some(rpc_msg) => self.handle_rpc_msg(rpc_msg.clone(), context.clone()),
            None => match msg.downcast_ref::<InsteonMsg>() {
                Some(insteon_msg) => self.handle_insteon_msg(insteon_msg.clone(), context.clone()),
                None => unreachable!(),
            }
        }

        //context.kill_me();
    }
}

pub struct RpcActor {
    pub ser_tx_actor : ActorRef,
    pub msg_bus      : Arc<Mutex<Bus<InsteonMsg>>>,
    pub event_loop   : Remote,
}

impl RpcActor {
    pub fn new(tuple: (ActorRef, Arc<Mutex<Bus<InsteonMsg>>>, Remote)) -> RpcActor {
        let (ser_tx_actor, msg_bus, event_loop) = tuple;
        RpcActor {
            ser_tx_actor: ser_tx_actor,
            msg_bus: msg_bus,
            event_loop: event_loop,
        }
    }

    pub fn handle_insteon_msg(&self, message: InsteonMsg, context: ActorCell) {
        info!("RpcActor: Received InsteonMsg: {:?}", message);
        for (_path, actor) in &context.children() {
            context.tell(actor.clone(), message);
        }
    }

    pub fn handle_rpc_msg(&self, message: RpcActorMsg, context: ActorCell) {

        //self.event_loop.execute()
        match message {
            RpcActorMsg::Set(light_control) => {
                let props = Props::new(Arc::new(RpcReqActor::new),
                                       (self.ser_tx_actor.clone(), self.msg_bus.clone(),
                                       self.event_loop.clone()));
                let req_actor = context.actor_of(props, "req_1".to_owned());
                info!("RpcActor received a message");
                context.tell(req_actor.unwrap(), RpcReqActorMsg::Set(
                    context.sender().clone(), light_control.clone()));
            },
            RpcActorMsg::SetReliable(light_control) => {
                let props = Props::new(Arc::new(RpcReqActor::new),
                                       (self.ser_tx_actor.clone(), self.msg_bus.clone(),
                                        self.event_loop.clone()));
                let req_actor = context.actor_of(props, "req_1".to_owned());
                info!("RpcActor received a message");
                context.tell(req_actor.unwrap(), RpcReqActorMsg::SetReliable(
                    context.sender().clone(), light_control.clone()));
            },
        }
    }
}

impl Actor for RpcActor {
    fn receive(&self, msg: Box<Any>, context: ActorCell) {
        match msg.downcast_ref::<RpcActorMsg>() {
            Some(rpc_msg) => self.handle_rpc_msg(rpc_msg.clone(), context),
            None => match msg.downcast_ref::<InsteonMsg>() {
                Some(insteon_msg) => self.handle_insteon_msg(insteon_msg.clone(), context),
                None => unreachable!(),
            }
        }
    }
}

#[derive(Clone)]
pub struct VinsteonRpcImpl {
    pub actor_system        : ActorSystem,
    pub rpc_actor           : ActorRef,
    pub ser_tx_actor        : ActorRef,
    pub msg_bus             : Arc<Mutex<Bus<InsteonMsg>>>
}

fn _log_result<T, E : Debug>(result: Result<T, E>) -> Result<(()), (())>{
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

pub const ACK_WAIT_INTERVAL_SEC : u64 = 1;
pub const SEND_RETRIES : usize = 8;

impl VinsteonRPC for VinsteonRpcImpl {

    fn send_cmd(&self, _m: grpc::RequestOptions, req: CmdMsg) -> grpc::SingleResponse<Ack> {

        let mut response = Ack::new();

        match req.cmd {
            Some(CmdMsg_oneof_cmd::lightControl(light_control)) => {
                let future = self.actor_system.ask(
                    self.rpc_actor.clone(),
                    RpcActorMsg::Set(light_control.clone()), "req_1".to_owned());
                response = self.actor_system.extract_result(future);
            }
            _ => error!("Unknown command"),
        }

        grpc::SingleResponse::completed(response)
    }

    fn send_cmd_reliable(&self, _m: grpc::RequestOptions, req: CmdMsg) -> grpc::SingleResponse<Ack> {
        let future = self.actor_system.ask(
            self.rpc_actor.clone(),
            RpcActorMsg::SetReliable(req.clone()), "req_1".to_owned());
        let response = self.actor_system.extract_result(future);

        grpc::SingleResponse::completed(response)
    }
}