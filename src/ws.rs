use futures_util::{SinkExt, StreamExt};
use crate::lnd::LndClient;
use log::{error, info};
use std::process::Command;
use std::sync::atomic::{AtomicU8, Ordering};
use std::{sync::Arc, time::Duration};
use tokio::sync::{
    mpsc::Receiver,
    Mutex,
};
use warp::filters::ws::Message;
use crate::config::Config;

pub async fn bartender_is_on_the_clock(
    serve: bool,
    ws: warp::ws::WebSocket,
    receiver: Arc<Mutex<Receiver<String>>>,
    recent_number: Arc<AtomicU8>,
) {
    let new_client_number = recent_number.load(Ordering::Relaxed) + 1;
    info!("New client connected: {}", new_client_number);
    recent_number.store(new_client_number, Ordering::Relaxed);

    let (mut bartender_tx, mut bartender_rx) = ws.split();

    let config = match Config::parse() {
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
