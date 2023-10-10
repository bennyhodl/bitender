use std::str::FromStr;
use lightning_invoice::Bolt11Invoice;
use tonic_lnd::{
    in_mem_connect,
    lnrpc::{InvoiceRequest, InvoiceSubscription, invoice::InvoiceState},
    Client,
};

pub struct LndClient {
    client: Client,
}

impl LndClient {
    pub async fn new(
        address: String,
        macaroon_hex: String,
        cert_hex: String,
    ) -> LndClient {
        let client = in_mem_connect(address, cert_hex, macaroon_hex).await;
        match client {
            Ok(client) => {
                println!("Connected to LND");
                LndClient { client }
            }
            Err(_e) => {
                panic!();
            }
        }
    }

    pub async fn stream_payments(&mut self) -> anyhow::Result<()> {
        println!("Streamin invoices!");
        let sub = InvoiceSubscription {
            add_index: 0,
            settle_index: 0
        };

        let mut subscription = self.client.lightning().subscribe_invoices(sub).await?.into_inner();

        while let Some(invoice) = subscription.message().await? {
            if let Some(state) = InvoiceState::from_i32(invoice.state) {
                if state == InvoiceState::Settled {
                    println!("Invoice settled! Amt: {}", invoice.amt_paid_msat);
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

        println!("Invoice {} created.", payment_hash);

        Ok(create_invoice_request.payment_request)
    }
    // pub async fn pay_invoice(&mut self, pay_req: String) -> anyhow::Result<String> {
    //     let pay_invoice_object = SendPaymentRequest {
    //         payment_request: pay_req.clone(),
    //         allow_self_payment: false,
    //         amp: false,
    //         timeout_seconds: 16,
    //         fee_limit_sat: 20,
    //     };
    //
    //     let payment_hash = Bolt11Invoice::from_str(&pay_req.as_str())?
    //         .payment_hash()
    //         .clone()
    //         .to_string();
    //
    //     info!("Paying invoice {}", payment_hash);
    //
    //     let mut pay_invoice_request = self
    //         .client
    //         .router()
    //         .send_payment_v2(pay_invoice_object)
    //         .await?
    //         .into_inner();
    //
    //     while let Ok(stream) = pay_invoice_request.message().await {
    //         match stream {
    //             Some(status) => match status.status {
    //                 0 => {
    //                     error!("Payment {} status unknown.", payment_hash);
    //                     return Err(anyhow!(failure_reason(status.failure_reason)));
    //                 }
    //                 1 => info!("Payment {} in-flight.", payment_hash),
    //                 2 => {
    //                     info!("Payment {} succeeded.", payment_hash);
    //                     return Ok(status.payment_hash);
    //                 }
    //                 3 => {
    //                     error!("Payment {} failed.", payment_hash);
    //                     return Err(anyhow!(failure_reason(status.failure_reason)));
    //                 }
    //                 _ => {
    //                     error!("Payment {} status failed/unknown.", payment_hash);
    //                     return Err(anyhow!(failure_reason(status.failure_reason)));
    //                 }
    //             },
    //             None => {
    //                 error!("Payment {} failed.", payment_hash);
    //                 return Err(anyhow!(failure_reason(6)));
    //             }
    //         }
    //     }
    //     error!(
    //         "Error in payment status stream for invoice {}",
    //         payment_hash
    //     );
    //     Err(anyhow!(""))
    // }
}

pub fn failure_reason(status: i32) -> String {
    match status {
        0 => format!("No failure."),
        1 => format!("Payment timed out."),
        2 => format!("No route for payment."),
        3 => format!("Error fulfilling payment."),
        4 => format!("Incorrect payment details."),
        5 => format!("Insufficient balance."),
        _ => format!("Failure reason unknown."),
    }
}
