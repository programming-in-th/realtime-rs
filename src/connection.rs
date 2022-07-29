use crate::channel::Channel;
use crate::error::Error;
use futures::channel::mpsc::{UnboundedReceiver, UnboundedSender};
use futures_util::{future, pin_mut, stream::SplitSink, stream::SplitStream, StreamExt};
use serde_json::{json, Value};
use std::{sync::Arc, time::Duration};
use tokio::{net::TcpStream, sync::Mutex, time::sleep};
use tokio_tungstenite::{
    connect_async, tungstenite::protocol::Message, MaybeTlsStream, WebSocketStream,
};
use url::Url;

pub struct SocketIO {
    pub socket_read: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    pub socket_write: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
    pub socket_tx: UnboundedSender<Message>,
    pub socket_rx: UnboundedReceiver<Message>,
}

pub struct Socket<'a> {
    pub socket_io: Option<SocketIO>,
    pub url: Url,
    pub connected: bool,
    pub channels: Vec<Channel<'a>>,
}

impl<'a> Socket<'a> {
    pub fn new(url: impl Into<String>) -> Self {
        Socket {
            socket_io: None,
            url: Url::parse(&url.into()).unwrap(),
            connected: false,
            channels: Vec::new(),
        }
    }

    pub async fn connect(&mut self) -> Result<(), Error> {
        let (stdin_tx, stdin_rx) = futures::channel::mpsc::unbounded::<Message>();

        let (ws_stream, _) = connect_async(&self.url).await?;
        println!("WebSocket handshake has been successfully completed");
        let (write, read) = ws_stream.split();

        self.socket_io = Some(SocketIO {
            socket_read: read,
            socket_write: write,
            socket_tx: stdin_tx,
            socket_rx: stdin_rx,
        });

        self.connected = true;
        Ok(())
    }

    pub fn set_channel(&mut self, topic: impl Into<String>) -> &mut Channel<'a> {
        let channel = Channel::new(topic, self.socket_io.as_ref().unwrap().socket_tx.clone());
        self.channels.push(channel);
        self.channels.last_mut().unwrap()
    }

    pub async fn listen(&mut self) {
        let socket_io = self.socket_io.take().unwrap();
        let (socket_rx, socket_tx, read, write) = (
            socket_io.socket_rx,
            socket_io.socket_tx,
            socket_io.socket_read,
            socket_io.socket_write,
        );

        let rx_to_ws = socket_rx.map(Ok).forward(write);

        let channels: Arc<Mutex<Vec<Channel>>> = Arc::new(Mutex::new(std::mem::replace(
            &mut self.channels,
            Vec::new(),
        )));

        // let channels = Rc::new(RefCell::new(&mut self.channels));
        let ws_to_cb = {
            read.for_each(|message| async {
                if let Ok(msg) = message {
                    if let Ok(data) = serde_json::from_str::<Value>(&msg.to_string()) {
                        if data["topic"].as_str().is_some()
                            && data["payload"].as_object().is_some()
                            && data["event"].as_str().is_some()
                        {
                            let topic = data["topic"].as_str().unwrap().to_string();
                            let event = data["event"].as_str().unwrap().to_string();
                            let payload = data["payload"].as_object().unwrap();

                            channels.lock().await.iter_mut().for_each(|channel| {
                                if channel.topic == topic {
                                    channel.listeners.iter_mut().for_each(|listener| {
                                        if listener.event == event || listener.event == "*" {
                                            listener.callback.as_mut()(&payload);
                                        }
                                    });
                                }
                            });
                        }
                    }
                }
            })
        };

        let awake = async move {
            let data = json!({
                "topic": "phoenix",
                "event": "heartbeat",
                "payload": {"msg": "ping"},
                "ref": null
            });
            while socket_tx
                .unbounded_send(Message::Text(data.to_string()))
                .is_ok()
            {
                sleep(Duration::from_secs(5)).await;
            }
        };

        tokio::spawn(awake);
        tokio::spawn(rx_to_ws);
        ws_to_cb.await;
    }
}
