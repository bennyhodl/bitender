pub mod config;
pub mod lnd;

use clap::Parser;
use futures_util::{SinkExt, StreamExt};
use lnd::LndClient;
use log::{error, info};
use std::process::Command;
use std::sync::atomic::{AtomicU8, Ordering};
use std::{sync::Arc, time::Duration};
use tokio::sync::{
    mpsc::{self, Receiver},
    Mutex,
};
use warp::{filters::ws::Message, Filter};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct BartenderArgs {
    /// Contract id from MongoDB that will be used by this instance.
    #[arg(short, long)]
    serve: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();
    info!("🎵 hey, bartender! 🎵");

    let args: BartenderArgs = clap::Parser::parse();

    match args.serve {
        true => info!("Bartender will be pouring drinks."),
        false => info!("Bartender WILL NOT be pouring drinks."),
    };

    let config = config::Config::parse()?;

    let mut client = LndClient::new(config.address, config.macaroon, config.cert).await;

    let (sender, receiver) = mpsc::channel::<String>(32);

    let receiver = Arc::new(Mutex::new(receiver));
    let sender = Arc::new(sender);

    let most_recent_number = Arc::new(AtomicU8::new(0));
    let ws_most_recent_number = warp::any().map(move || most_recent_number.clone());

    tokio::spawn(async move {
        let _ = client.stream_payments(sender.clone()).await;
    });

    let routes = warp::path("bartender")
        .and(warp::ws())
        .and(ws_most_recent_number.clone())
        .map(move |ws: warp::ws::Ws, recent_number: Arc<AtomicU8>| {
            let receiver = receiver.clone();
            ws.on_upgrade(move |websocket| {
                bartender_is_on_the_clock(args.serve, websocket, receiver, recent_number.clone())
            })
        });

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;

    Ok(())
}

async fn bartender_is_on_the_clock(
    serve: bool,
    ws: warp::ws::WebSocket,
    receiver: Arc<Mutex<Receiver<String>>>,
    recent_number: Arc<AtomicU8>,
) {
    let new_client_number = recent_number.load(Ordering::Relaxed) + 1;
    info!("New client connected: {}", new_client_number);
    recent_number.store(new_client_number, Ordering::Relaxed);

    let (mut bartender_tx, mut bartender_rx) = ws.split();

    let config = match config::Config::parse() {
        Ok(c) => c,
        Err(_) => panic!("No env!"),
    };

    let pr_receiver = receiver.clone();

    tokio::spawn(async move {
        let mut client = LndClient::new(config.address, config.macaroon, config.cert).await;

        let pay_req = match client
            .create_invoice("Bitcoin Bay Bartender".to_string(), 50)
            .await
        {
            Ok(pr) => pr,
            Err(_e) => "no-payment-request".to_string(),
        };

        while let Some(_message) = bartender_rx.next().await {
            let _ = bartender_tx.send(Message::text(pay_req.clone())).await;

            while let Some(msg) = pr_receiver.lock().await.recv().await {
                if let Err(e) = bartender_tx.send(Message::text(msg)).await {
                    error!("Error: {}", e);
                };

                if serve {
                    let _pour_beer = Command::new("python3")
                        .arg("gpio_handler.py")
                        .arg("-p")
                        .arg("cocktail")
                        .status();
                }

                let next_pr = match client
                    .create_invoice("Bitcoin Bay Bartender".to_string(), 50)
                    .await
                {
                    Ok(pr) => pr,
                    Err(e) => {
                        error!("Error creating new payment request: {}", e);
                        continue;
                    }
                };

                let _pour_beer = tokio::time::sleep(Duration::from_secs(5)).await;

                let _send_new_pr = bartender_tx.send(Message::text(next_pr)).await;
            }
        }
    });
}
