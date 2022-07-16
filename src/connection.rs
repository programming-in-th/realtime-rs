use crate::channel::Channel;
use crate::error::Error;
use serde_json::{Value};
use std::net::TcpStream;
use tungstenite::{connect, stream::MaybeTlsStream, Message, WebSocket};
use url::Url;

pub struct Socket {
    pub socket: Option<WebSocket<MaybeTlsStream<TcpStream>>>,
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
        self.socket = Some(connect(&self.url)?.0);
        self.connected = true;
        Ok(())
    }

    pub fn set_channel(&mut self, topic: impl Into<String>) -> &mut Channel {
        let channel = Channel::new(topic);
        self.channels.push(channel);
        self.channels.last_mut().unwrap()
    }

    pub fn listen(&mut self) -> Result<(), Error> {
        self._join()?;
        self._listen();
        Ok(())
    }

    fn _listen(&mut self) {
        loop {
            let msg = self.socket.as_mut().unwrap().read_message();
            if msg.is_ok() {
                let msg = msg.unwrap();
                if let Ok(data) = serde_json::from_str::<Value>(&msg.to_string()) {
                    let topic = match data["topic"].as_str() {
                        Some(topic) => topic.to_string(),
                        None => continue,
                    };
                    let event = match data["event"].as_str() {
                        Some(event) => event,
                        None => continue,
                    };
                    let payload = match data["payload"].as_object() {
                        Some(topic) => topic,
                        None => continue,
                    };
                    for channel in self.channels.iter_mut() {
                        if channel.topic == topic {
                            for listener in channel.listeners.iter_mut() {
                                if listener.event == event {
                                    listener.callback.as_mut()(payload);
                                }
                            }
                        }
                    }
                }
            } else {
                println!("{:?}", msg.unwrap_err());
                break;
            }
        }
    }

    // async fn _keep_alive(&mut self) {
    //     loop {
    //         let data = json!({
    //             "topic": "phoenix",
    //             "event": "heartbeat",
    //             "payload": {"msg": "ping"},
    //             "ref": null
    //         });
    //         let msg = Message::Text(data.to_string());
    //         if self.socket.as_mut().unwrap().write_message(msg).is_err() {
    //             break;
    //         }
    //         task::sleep(Duration::from_secs(1)).await;
    //     }
    // }

    fn _join(&mut self) -> Result<(), Error> {
        for channel in self.channels.iter() {
            self.socket
                .as_mut()
                .unwrap()
                .write_message(Message::Text(channel.get_join()))?;
        }
        Ok(())
    }
}
