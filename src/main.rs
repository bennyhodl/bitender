pub mod config;
pub mod lnd;

use futures_util::{SinkExt, StreamExt};
use lnd::LndClient;
use log::{error, info};
use std::process::Command;
use std::{sync::Arc, time::Duration};
use tokio::sync::{
    mpsc::{self, Receiver},
    Mutex,
};
use warp::{filters::ws::Message, Filter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();
    info!("🎵 hey, bartender! 🎵");

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
            ws.on_upgrade(move |websocket| bartender_is_on_the_clock(websocket, receiver))
        });

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;

    Ok(())
}

async fn bartender_is_on_the_clock(
    ws: warp::ws::WebSocket,
    receiver: Arc<Mutex<Receiver<String>>>,
) {
    let (mut bartender_tx, mut bartender_rx) = ws.split();

    let config = match config::Config::parse() {
        Ok(c) => c,
        Err(_) => panic!("No env!"),
    };

    tokio::spawn(async move {
        let receiver_clone = receiver.clone();

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

            while let Some(msg) = receiver_clone.lock().await.recv().await {
                if let Err(e) = bartender_tx.send(Message::text(msg)).await {
                    error!("Error: {}", e);
                };

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

                // Send raspi scrip here?
                let pour_beer = Command::new("python3")
                    .arg("gpio_handler.py")
                    .arg("-p cocktail")
                    .output();

                match pour_beer {
                    Ok(output) => {
                        if output.status.success() {
                            // The command was successful.
                            let stdout = String::from_utf8_lossy(&output.stdout);
                            print!("Command output:\n{}", stdout);
                        } else {
                            eprintln!("Command failed with error code: {:?}", output.status);
                        }
                    }
                    Err(err) => {
                        eprintln!("Error executing command: {:?}", err);
                    }
                }

                let _pour_beer = tokio::time::sleep(Duration::from_secs(5)).await;

                let _send_new_pr = bartender_tx.send(Message::text(next_pr)).await;
            }
        }
    });
}
