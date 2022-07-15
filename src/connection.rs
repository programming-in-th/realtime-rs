use crate::error::Error;
use crate::channel::Channel;
use std::net::TcpStream;
use tungstenite::{connect, stream::MaybeTlsStream, WebSocket};
use url::Url;

pub struct Socket {
    pub socket: Option<WebSocket<MaybeTlsStream<TcpStream>>>,
    pub url: Url,
    pub connected: bool,
    pub channels: Vec<Channel>,
}

impl Socket {
    pub fn new(url: impl Into<String>) -> Result<Self, Error> {
        Ok(Socket {
            socket: None,
            url: Url::parse(&url.into())?,
            connected: false,
            channels: Vec::new(),
        })
    }

    pub fn connect(&mut self) -> Result<(), Error> {
        self.socket = Some(connect(&self.url)?.0);
        self.connected = true;
        Ok(())
    }

    pub fn set_channel<T: Into<String>>(&mut self, topic: impl Into<String>) -> Result<&mut Channel, Error> {
        let channel = Channel::new(topic)?;
        self.channels.push(channel);
        Ok(self.channels.last_mut().unwrap())
    }
}
