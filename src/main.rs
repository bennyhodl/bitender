pub mod lnd;
pub mod config;

use std::sync::Arc;

use futures_util::{SinkExt, StreamExt};
use lnd::LndClient;
use tokio::sync::{Mutex, mpsc::{self, Receiver}};
use warp::{Filter, filters::ws::Message};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🎵 hey, bartender! 🎵");
    
    let config = config::Config::parse()?;

    let mut client = LndClient::new(config.address, config.macaroon, config.cert).await; 

    let (sender, receiver) = mpsc::channel::<String>(32);

    let receiver = Arc::new(Mutex::new(receiver));
    let sender = Arc::new(sender);

    tokio::spawn(async move {
        let _ = client.stream_payments(sender.clone()).await;
    });
    
    let routes = warp::path("bartender")
        .and(warp::ws())
        .map(move |ws: warp::ws::Ws| {
            let receiver = receiver.clone();
            ws.on_upgrade(move |websocket| {
                bartender_is_alive(websocket, receiver)
            })
        });

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;

    Ok(())
}

async fn bartender_is_alive(ws: warp::ws::WebSocket, receiver: Arc<Mutex<Receiver<String>>>) {
    let (mut bartender_tx, mut bartender_rx) = ws.split();

    let config = match config::Config::parse() {
        Ok(c) => c,
        Err(_) => panic!("No env!")
    };

    let mut client = LndClient::new(config.address, config.macaroon, config.cert).await; 
    
    let pay_req = client.create_invoice("heyhowareya".to_string(), 50_000).await.unwrap();

    // let bartender_tx_mu = Arc::new(Mutex::new(bartender_tx));

    tokio::spawn(async move {
        let receiver_clone = receiver.clone();
        while let Some(_message) = bartender_rx.next().await {
            // let mut tx = bartender_tx.lock().await;
            let _ = bartender_tx.send(Message::text(pay_req.clone())).await;

            while let Some(msg) = receiver_clone.lock().await.recv().await {
                println!("Got message across you biotch {}", msg);
                // let mut tx = bartender_tx.lock().await;
                if let Err(e) = bartender_tx.send(Message::text(msg)).await {
                    eprintln!("Error: {}", e);
                };
            };
        }
    });
}
