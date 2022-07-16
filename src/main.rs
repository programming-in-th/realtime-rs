use realtime_rs::connection::Socket;
fn main() {
    let url = "ws://localhost:4000/socket/websocket";
    let mut socket = Socket::new(url);
    socket.connect().unwrap();

    let channel = socket.set_channel("realtime:public");

    channel.join().on(
        "INSERT",
        Box::new(|data| {
            println!("result: {:?}", data);
        }),
    );

    socket.listen();
}
