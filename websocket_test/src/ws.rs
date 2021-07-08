use actix::{fut, ActorContext};
use crate::messages::{Disconnect, Connect, WsMessage, ClientActorMessage};
use crate::lobby::Lobby;
use actix::{Actor, Addr, Running, StreamHandler, WrapFuture, ActorFuture, ContextFutureSpawner};
use actix::{AsyncContext, Handler};
use actix_web_actors::ws;
use actix_web_actors::ws::Message::Text;
use std::time::{Duration, Instant};
use uuid::Uuid;

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

pub struct WsConn {
    room: Uuid,
    lobby_addr: Addr<Lobby>,
    hb: Instant, //heartbeat: moment in time
    id: Uuid
}

impl WsConn {
    pub fn new(room: Uuid, lobby: Addr<Lobby>) -> WsConn {
        WsConn {
            id: Uuid::new_v4(),
            room,
            hb: Instant::now(),
            lobby_addr: lobby,
        }
    }
}

impl Actor for WsConn {
    type Context = ws::WebsocketContext<Self>;
    
    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);
        println!("Started!");
        let addr = ctx.address();
        //send connect message
        self.lobby_addr.send(Connect {
            addr: addr.recipient(),
            lobby_id: self.room,
            self_id: self.id
        })
        .into_actor(self)
        .then(|res, _, ctx| {
            match res {
                Ok(_res) => (),
                _ => ctx.stop()
            }
            fut::ready(())
        })
        .wait(ctx);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        self.lobby_addr.do_send(Disconnect { 
            id: self.id,
            room_id: self.room, 
        });
        Running::Stop
    }
}


//heartbeat method
impl WsConn {
    
    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        println!("HB");
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                println!("disconnecting due to failed heartbeat");
                act.lobby_addr.do_send(Disconnect {
                    id: act.id, room_id: act.room
                });
                ctx.stop();
                return;
            } 

            ctx.ping(b"PING");
        });
    }
}

//Handle different msg types and err case
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsConn {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                println!("Ping");
                self.hb = Instant::now();
                ctx.pong(&msg);
            } 
            Ok(ws::Message::Pong(_)) => {
                self.hb = Instant::now();
            }
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();

            }
            Ok(ws::Message::Continuation(_)) => {
                println!("Continuation");
                ctx.stop();
            }
            Ok(ws::Message::Nop) => (),
            Ok(Text(s)) => self.lobby_addr.do_send(ClientActorMessage {
                id: self.id,
                msg: s,
                room_id: self.room
            }),
            Err(e) => panic!(e),
        }
    }
}

impl Handler<WsMessage> for WsConn {
    type Result = ();
    
    fn handle(&mut self, msg:WsMessage, ctx: &mut Self::Context) {
        println!("Ganzunten ");
        ctx.text(msg.0);
    }
}