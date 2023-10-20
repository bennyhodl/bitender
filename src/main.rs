mod config;
mod lnd;
mod ws;

use clap::Parser;
use lnd::LndClient;
use log::info;
use std::sync::atomic::AtomicU8;
use std::sync::Arc;
use tokio::sync::{
    mpsc,
    Mutex,
};
use warp::Filter;
use ws::bartender_is_on_the_clock;

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

