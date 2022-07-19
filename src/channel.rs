use crate::error::Error;
use futures::channel::mpsc::UnboundedSender;
use serde_json::{json, Map, Value};
use std::{
    net::TcpStream,
    sync::{Arc, Mutex},
};
use tungstenite::{stream::MaybeTlsStream, Message, WebSocket};

type Callback<'a> = Box<dyn Fn(&Map<String, Value>) + Send + 'a>;
pub type SocketModule = Arc<Mutex<WebSocket<MaybeTlsStream<TcpStream>>>>;

pub struct CallBackListener<'a> {
    pub callback: Callback<'a>,
    pub event: String,
}

impl<'a> CallBackListener<'a> {
    pub fn new(callback: Callback<'a>, event: impl Into<String>) -> Self {
        CallBackListener {
            callback,
            event: event.into(),
        }
    }
}

fn generate_json(topic: &str) -> String {
    let json = json!({
        "topic": topic,
        "event": "phx_join",
        "payload": {},
        "ref": null
    });
    return json.to_string();
}

pub struct Channel<'a> {
    pub socket: UnboundedSender<Message>,
    pub listeners: Vec<CallBackListener<'a>>,
    pub topic: String,
}

impl<'a> Channel<'a> {
    pub fn new(topic: impl Into<String>, socket: UnboundedSender<Message>) -> Self {
        Channel {
            socket,
            listeners: Vec::new(),
            topic: topic.into(),
        }
    }

    pub fn join(&mut self) -> &mut Self {
        let json = generate_json(&self.topic);
        self.socket.unbounded_send(Message::Text(json)).unwrap();
        self
    }

    pub fn on(&mut self, event: impl Into<String>, callback: Callback<'a>) -> &mut Self {
        self.listeners
            .push(CallBackListener::new(callback, event.into()));
        self
    }

    pub fn off(&mut self, event: impl Into<String>) -> Result<&mut Self, Error> {
        let event = event.into();
        let index = self
            .listeners
            .iter()
            .position(|l| l.event == event)
            .ok_or(Error::NotFoundError)?;
        self.listeners.remove(index);
        Ok(self)
    }

    pub fn get_join(&self) -> String {
        generate_json(&self.topic)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn generate_json_correctly() {
        let topic = "random";
        let json = generate_json(topic);
        let expected = "{\"event\":\"phx_join\",\"payload\":{},\"ref\":null,\"topic\":\"random\"}";
        assert_eq!(json, expected);
    }
}
