use actix::{Message, Recipient};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Message)]
#[rtype(result = "()")]
pub struct MyMessage(pub String);

#[derive(Message)]
#[rtype(result = "()")]
pub struct Connect {
    pub id: Uuid,
    pub addr: Recipient<MyMessage>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation<T> {
    pub kind: String,
    pub data: T,
}

impl<T> Conversation<T> {
    pub fn new(kind: String, data: T) -> Self {
        Self {
            kind,
            data,
        }
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct WrappedConversation<T>(pub Uuid, pub Conversation<T>);