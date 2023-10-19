use lightning_invoice::Bolt11Invoice;
use log::info;
use std::{str::FromStr, sync::Arc};
use tokio::sync::mpsc::Sender;
use tonic_lnd::{
    in_mem_connect,
    lnrpc::{invoice::InvoiceState, InvoiceRequest, InvoiceSubscription},
    Client,
};

type LndSender = Arc<Sender<String>>;

pub struct LndClient {
    client: Client,
}

impl LndClient {
    pub async fn new(address: String, macaroon_hex: String, cert_hex: String) -> LndClient {
        info!("Calling the bartender.");
        let client = in_mem_connect(address, cert_hex, macaroon_hex).await;
        match client {
            Ok(client) => {
                info!("The bar is now open.");
                LndClient { client }
            }
            Err(e) => {
                info!("Error LND! {}", e);
                panic!();
            }
        }
    }

    pub async fn stream_payments(&mut self, sender: LndSender) -> anyhow::Result<()> {
        let sub = InvoiceSubscription {
            add_index: 0,
            settle_index: 0,
        };

        let mut subscription = self
            .client
            .lightning()
            .subscribe_invoices(sub)
            .await?
            .into_inner();

        while let Some(invoice) = subscription.message().await? {
            if let Some(state) = InvoiceState::from_i32(invoice.state) {
                if state == InvoiceState::Settled
                    && invoice.memo == "Bitcoin Bay Bartender".to_string()
                {
                    info!("Bar tab paid. Pouring a beer...");
                    let _ = sender
                        .send("hey-bartender-pour-me-a-beer".to_string())
                        .await;
                }
            }
        }

        Ok(())
    }

    pub async fn create_invoice(&mut self, memo: String, value: i64) -> anyhow::Result<String> {
        let create_invoice_object = InvoiceRequest {
            memo,
            value_msat: value * 1_000,
            private: true,
            is_amp: false,
            is_keysend: false,
        };

        let create_invoice_request = self
            .client
            .lightning()
            .add_invoice(create_invoice_object)
            .await?
            .into_inner();

        let payment_hash =
            Bolt11Invoice::from_str(create_invoice_request.payment_request.as_str())?
                .payment_hash()
                .clone()
                .to_string();

        info!("Bar tab created: {}", payment_hash);

        Ok(create_invoice_request.payment_request)
    }
}
