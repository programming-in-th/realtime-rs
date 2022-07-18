use realtime_rs::connection::Socket;

#[tokio::main]
async fn main() {
    let url = "ws://localhost:4000/socket/websocket";
    let mut socket = Socket::new(url);
    socket.connect().await.unwrap();
    let channel = socket.set_channel("realtime:public");

    channel.join().on(
        "INSERT",
        Box::new(|data| {
            println!("result: {:?}", data);
        }),
    );

    socket.listen().await;
}
