use crate::channel::{Channel, SocketModule};
use crate::error::Error;
use serde_json::{json, Value};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;
use tungstenite::{connect, Message};
use url::Url;

pub struct Socket {
    pub socket: Option<SocketModule>,
    pub url: Url,
    pub connected: bool,
    pub channels: Vec<Channel>,
}

impl Socket {
    pub fn new(url: impl Into<String>) -> Self {
        Socket {
            socket: None,
            url: Url::parse(&url.into()).unwrap(),
            connected: false,
            channels: Vec::new(),
        }
    }

    pub fn connect(&mut self) -> Result<(), Error> {
        self.socket = Some(Arc::new(Mutex::new(connect(&self.url)?.0)));
        self.connected = true;
        Ok(())
    }

    pub fn set_channel(&mut self, topic: impl Into<String>) -> &mut Channel {
        let channel = Channel::new(topic, self.socket.as_ref().unwrap());
        self.channels.push(channel);
        self.channels.last_mut().unwrap()
    }

    pub fn listen(&mut self) {
        let socket_listen = Arc::clone(self.socket.as_ref().unwrap());
        let socket_keep_alive = Arc::clone(self.socket.as_ref().unwrap());

        let (tx, rx) = mpsc::channel();

        let keep_alive = thread::spawn(move || {
            let data = json!({
                "topic": "phoenix",
                "event": "heartbeat",
                "payload": {"msg": "ping"},
                "ref": null
            });

            loop {
                if let Ok(mut socket) = socket_keep_alive.try_lock() {
                    socket
                        .write_message(Message::Text(data.to_string()))
                        .unwrap();
                    println!("sent heartbeat");
                    thread::sleep(Duration::from_secs(1));
                }
            }
        });

        let listen = thread::spawn(move || loop {
            if let Ok(mut msg) = socket_listen.try_lock() {
                if msg.can_read() {
                    let msg = msg.read_message().unwrap();
                    if let Ok(data) = serde_json::from_str::<Value>(&msg.to_string()) {
                        let topic = match data["topic"].as_str() {
                            Some(topic) => topic.to_string(),
                            None => continue,
                        };
                        let event = match data["event"].as_str() {
                            Some(event) => event.to_string(),
                            None => continue,
                        };
                        let payload = match data["payload"].as_object() {
                            Some(topic) => topic.clone(),
                            None => continue,
                        };
                        tx.send((topic, event, payload)).unwrap();
                    }
                } else {
                    println!("can't read");
                }
            }
        });

        for (topic, event, payload) in rx {
            for channel in self.channels.iter_mut() {
                if channel.topic == topic {
                    for listener in channel.listeners.iter_mut() {
                        if listener.event == event {
                            listener.callback.as_mut()(&payload);
                        }
                    }
                }
            }
        }

        keep_alive.join().unwrap();
        listen.join().unwrap();
    }
}
