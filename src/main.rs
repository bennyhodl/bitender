use futures_util::{SinkExt, StreamExt, TryFutureExt};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::Filter;

#[tokio::main]
async fn main() {
    println!("🎵 hey, bartender! 🎵");

    let routes = warp::path("bartender")
        .and(warp::ws())
        .map(|ws: warp::ws::Ws| ws.on_upgrade(|websocket| bartender_is_alive(websocket)));

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

async fn bartender_is_alive(ws: warp::ws::WebSocket) {
    let (mut bartender_tx, mut bartender_rx) = ws.split();

    // let (_tx, rx) = mpsc::unbounded_channel();

    // let mut rx = UnboundedReceiverStream::new(rx);

    tokio::task::spawn(async move {
        while let Some(message) = bartender_rx.next().await {
            match bartender_tx.send(message.unwrap()).await {
                Ok(_msg) => println!("Message sent"),
                Err(_e) => eprintln!("Message not send")
            }
        }
    });
}
