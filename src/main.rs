pub mod lnd;

use futures_util::{SinkExt, StreamExt};
// use tokio::sync::mpsc;
use warp::{Filter, filters::ws::Message};
use dotenv::dotenv;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🎵 hey, bartender! 🎵");

    dotenv().ok();

    let macaroon = std::env::var("MACAROON").unwrap();
    let cert = std::env::var("CERT").unwrap();
    let address = std::env::var("ADDRESS").unwrap();

    let mut client = lnd::LndClient::new(address, macaroon, cert).await; 

    tokio::spawn(async move {
        println!("Spawned process.");
        let _ = client.stream_payments().await;
    });
    
    let routes = warp::path("bartender")
        .and(warp::ws())
        .map(|ws: warp::ws::Ws| ws.on_upgrade(|websocket| bartender_is_alive(websocket)));

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;

    Ok(())
}

async fn bartender_is_alive(ws: warp::ws::WebSocket) {
    let (mut bartender_tx, mut bartender_rx) = ws.split();

    // let (_tx, rx) = mpsc::unbounded_channel();

    // let mut rx = UnboundedReceiverStream::new(rx);

    tokio::task::spawn(async move {
        while let Some(_message) = bartender_rx.next().await {
            match bartender_tx.send(Message::text("Thisisbolt11")).await {
                Ok(_msg) => println!("Message sent"),
                Err(_e) => eprintln!("Message not send")
            }
        }
    });
}
