use openlimits::binance::Binance;
use openlimits::binance::BinanceParameters;
use openlimits::coinbase::Coinbase;
use openlimits::coinbase::CoinbaseParameters;
use openlimits::exchange::Exchange;

#[derive(Clone)]
pub struct Client {
    pub binance: Binance,
    pub coinbase: Coinbase
}

impl Client {
    pub async fn new() -> Self {
        Self {
            binance: Binance::new(BinanceParameters::prod())
                        .await
                        .expect("Couldn't create binance client"),

            coinbase: Coinbase::new(CoinbaseParameters::prod())
                        .await
                        .expect("Couldn't create coinbase client"),
        }
    }
}