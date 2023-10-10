use dotenv::dotenv;

pub struct Config {
    pub macaroon: String,
    pub cert: String,
    pub address: String
}

impl Config {
    pub fn parse() -> anyhow::Result<Config> {
        dotenv().ok();

        let macaroon = std::env::var("MACAROON").unwrap();
        let cert = std::env::var("CERT").unwrap();
        let address = std::env::var("ADDRESS").unwrap();

        Ok(Config {
            macaroon,
            cert,
            address
        })
    }
}
