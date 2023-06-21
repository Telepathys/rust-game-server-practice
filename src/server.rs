use std::time::{Duration, Instant};
use actix::prelude::*;
use actix_web_actors::ws;
use uuid::Uuid;
use crate::game::Game;
use crate::message::{Connect, Conversation, Disconnect, MyMessage, WrappedConversation};

pub struct Session {
    id: Uuid,
    bz: Instant,
    addr: Addr<Game>,
}

impl Session {
    pub fn new(game: Addr<Game>) -> Self {
        Self {
            id: Uuid::new_v4(),
            bz: Instant::now(),
            addr: game
        }
    }

    fn bz(&self, ctx: &mut <Self as Actor>::Context) {
        ctx.run_interval(Duration::from_secs(5), |act, ctx| {
            if Instant::now().duration_since(act.bz) > Duration::from_secs(10) {
                ctx.stop();
                return;
            }

            ctx.ping(b"");
        });
    }
}

impl Actor for Session {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.bz(ctx);

        let addr = ctx.address();

        self.addr.send(Connect {
            id: self.id,
            addr: addr.recipient(),
        })
            .into_actor(self)
            .then(|res, _act, ctx| {
                match res {
                    Ok(_) => {},
                    _ => ctx.stop()
                }
                fut::ready(())
            })
            .wait(ctx);

    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        self.addr.do_send(Disconnect {
            id: self.id,
        });
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for Session {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.bz = Instant::now();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                self.bz = Instant::now();
            }
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            Ok(ws::Message::Text(s)) => {
                let conversation = match serde_json::from_str::<Conversation<Vec<f32>>>(&s) {
                    Ok(conversation) => conversation,
                    Err(e) => {
                        println!("Error: {}", e);
                        return;
                    }
                };

                self.addr.do_send(WrappedConversation(self.id, conversation));
            }
            _ => {}
        }
    }
}

impl Handler<MyMessage> for Session {
    type Result = ();

    fn handle(&mut self, msg: MyMessage, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}