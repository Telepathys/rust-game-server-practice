use std::any::Any;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime};
use actix::{Actor, ActorContext, AsyncContext, Context, Handler, Recipient};
use serde::{Serialize};
use uuid::Uuid;
use crate::geometry::vector::Vector2f;
use crate::message::{Connect, Conversation, Disconnect, MyMessage, WrappedConversation};

#[typetag::serialize(tag = "kind")]
pub trait Entity {
    fn update(&mut self, delta: f32);

    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;
}

#[derive(Debug, Clone, Serialize)]
pub struct Bullet {
    pub id: Uuid,
    pub owner: Option<Uuid>,
    pub position: Vector2f,
    pub velocity: Vector2f,
}

impl Bullet {
    pub fn new(owner: Option<Uuid>, position: Vector2f, velocity: Vector2f) -> Self {
        Self {
            id: Uuid::new_v4(),
            owner,
            position,
            velocity,
        }
    }
}

#[typetag::serialize]
impl Entity for Bullet {
    fn update(&mut self, delta: f32) {
        self.position.x += self.velocity.x * delta;
        self.position.y += self.velocity.y * delta;

    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Player {
    pub id: Uuid,
    pub health: f32,
    pub position: Vector2f,
    pub velocity: Vector2f,
}

impl Player {
    pub fn new(id: Uuid) -> Self {
        Self {
            id,
            health: 100.0,
            position: Vector2f::new(fastrand::f32() * 800.0, fastrand::f32() * 600.0),
            velocity: Vector2f::new(0.0, 0.0),
        }
    }
}

#[typetag::serialize]
impl Entity for Player {
    fn update(&mut self, delta: f32) {
        self.position.x += self.velocity.x * delta;
        self.position.y += self.velocity.y * delta;

        if self.position.x < 0.0 {
            self.position.x = 0.0;
            self.velocity.x *= -0.8;
        } else if self.position.x > 800.0 {
            self.position.x = 800.0;
            self.velocity.x *= -0.8;
        }

        if self.position.y < 0.0 {
            self.position.y = 0.0;
            self.velocity.y *= -0.8;
        } else if self.position.y > 600.0 {
            self.position.y = 600.0;
            self.velocity.y *= -0.8;
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[derive(Serialize)]
pub struct GameState {
    pub ts: i64,
    pub entities: HashMap<Uuid, Box<dyn Entity>>,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            ts: chrono::Utc::now().timestamp_millis(),
            entities: HashMap::new(),
        }
    }
}

type Session = Recipient<MyMessage>;

pub struct Game {
    state: Arc<Mutex<GameState>>,
    sessions: HashMap<Uuid, Session>,
    start_time: Instant,
}

impl Default for Game {
    fn default() -> Self {
        Self {
            state: Arc::new(Mutex::new(GameState::default())),
            sessions: HashMap::new(),
            start_time: Instant::now(),
        }
    }
}

impl Game {
    fn notify<T: Serialize>(&self, conversation: Conversation<T>) {
        let msg = match serde_json::to_string(&conversation) {
            Ok(msg) => msg,
            Err(_) => return,
        };

        for (_, addr) in self.sessions.iter() {
            let _ = addr.do_send(MyMessage(msg.clone()));
        }
    }

    fn start_ticker(&mut self, ctx: &mut <Self as Actor>::Context) {
        self.start_time = Instant::now();

        ctx.run_interval(Duration::from_millis(16), |act, _ctx| {
            let current_time = Instant::now();
            let delta = current_time.duration_since(act.start_time).as_secs_f32();
            act.start_time = current_time;

            let mut state = match act.state.lock() {
                Ok(state) => state,
                Err(_) => return,
            };

            for entity in state.entities.values_mut() {
                entity.update(delta);
            }

            state.ts = chrono::Utc::now().timestamp_millis();

            let data = match serde_json::to_string(&state.deref()) {
                Ok(msg) => msg,
                Err(_) => return,
            };

            let conversation = Conversation::new("game_state".to_string(), data);
            act.notify(conversation);
        });
    }
}

impl Actor for Game {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.start_ticker(ctx);
    }
}

impl Handler<Connect> for Game {
    type Result = ();

    fn handle(&mut self, msg: Connect, ctx: &mut Self::Context) -> Self::Result {
        let mut state = match self.state.lock() {
            Ok(state) => state,
            Err(_) => {
                ctx.stop();
                return;
            },
        };

        self.sessions.insert(msg.id, msg.addr);
        state.entities.insert(msg.id, Box::new(Player::new(msg.id)));
    }
}

impl Handler<Disconnect> for Game {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, ctx: &mut Self::Context) -> Self::Result {
        let mut state = match self.state.lock() {
            Ok(state) => state,
            Err(_) => {
                ctx.stop();
                return;
            }
        };

        self.sessions.remove(&msg.id);
        state.entities.remove(&msg.id);
    }
}

impl Handler<WrappedConversation<Vec<f32>>> for Game {
    type Result = ();

    fn handle(&mut self, msg: WrappedConversation<Vec<f32>>, ctx: &mut Self::Context) -> Self::Result {
        let mut state = match self.state.lock() {
            Ok(state) => state,
            Err(_) => {
                ctx.stop();
                return;
            }
        };

        let kind = msg.1.kind.as_str();

        match kind {
            "move" => {
                let entity = match state.entities.get_mut(&msg.0) {
                    Some(entity) => entity,
                    None => return,
                };

                let player = match entity.as_any_mut().downcast_mut::<Player>() {
                    Some(player) => player,
                    None => return,
                };

                player.velocity.x += msg.1.data[0];
                player.velocity.y += msg.1.data[1];
            }
            "fire" => {
                let entity = match state.entities.get(&msg.0) {
                    Some(entity) => entity,
                    None => return,
                };

                let player = match entity.as_any().downcast_ref::<Player>() {
                    Some(player) => player,
                    None => return,
                };

                let click_pos = Vector2f::new(msg.1.data[0], msg.1.data[1]);
                let player_pos = player.position.clone();
                let angle = (click_pos - player_pos.clone()).angle();
                let velocity = Vector2f::from_angle(angle);

                state.entities.insert(
                    Uuid::new_v4(),
                    Box::new(Bullet::new(Some(msg.0), player_pos, velocity * 300.0)),
                );
            }
            _ => {}
        };
    }
}