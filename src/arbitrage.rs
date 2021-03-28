use crate::client::Client;
use crate::exchange::Exchange;
use crate::pair::Pair;
use openlimits::exchange::ExchangeMarketData;
use openlimits::model::OrderBookRequest;
use openlimits::model::OrderBookResponse;
use chrono::Local;
use stopwatch::Stopwatch;
use rust_decimal::Decimal;

//a parser trait for OderBookRequest.
trait Parser {
    fn parse(client: Exchange, pair: &Pair) -> OrderBookRequest;
}

//implements parser trait.
impl Parser for OrderBookRequest {
    fn parse(client: Exchange, pair: &Pair) -> OrderBookRequest {
        match &client {
            Exchange::Binance => match pair {
                Pair::BtcEur => OrderBookRequest {market_pair: "BTCEUR".to_string()},
                Pair::EthEur => OrderBookRequest {market_pair: "ETHEUR".to_string()},
                Pair::BtcGbp => OrderBookRequest {market_pair: "BTCGBP".to_string()},
            },
            Exchange::Coinbase => match pair {
                Pair::BtcEur => OrderBookRequest {market_pair: "BTC-EUR".to_string()},
                Pair::EthEur => OrderBookRequest {market_pair: "ETH-EUR".to_string()},
                Pair::BtcGbp => OrderBookRequest {market_pair: "BTC-GBP".to_string()},
            }
        }
    }
}

#[derive(Clone)]
pub struct Arbitrage {
    client: Client,
    binance_bid: Decimal,
    binance_ask: Decimal,
    coinbase_bid: Decimal,
    coinbase_ask: Decimal,
    pair: Pair,
}

impl Arbitrage {
    pub async fn new(pair: Pair) -> Self {
        Self {
            client: Client::new().await,
            binance_bid: Decimal::default(),
            binance_ask: Decimal::default(),
            coinbase_bid: Decimal::default(),
            coinbase_ask: Decimal::default(),
            pair
        }
    }
    
    async fn get_order_book(&self, client: Exchange) -> OrderBookResponse {
        match &client {
            Exchange::Binance => self.client
                                            .binance
                                            .order_book(&OrderBookRequest::parse(client, &self.pair))
                                            .await
                                            .expect("Couldn't get coinbase order book"),

            Exchange::Coinbase => self.client
                                            .coinbase.order_book(&OrderBookRequest::parse(client, &self.pair))
                                            .await
                                            .expect("Couldn't get coinbase order book")
        }
    }

    async fn update_prices(&mut self) {
        //cloning self to be able to use on the tokio spawn.
        let self_ = self.clone();
        //gets the binance order book.
        let binance_order_book = tokio::spawn(async move { 
            self_.get_order_book(Exchange::Binance)
                .await 
        })
        .await
        .expect("Couldn't get binance order book");
        
        //gets the coinbase order book.
        let coinbase_order_book = self.get_order_book(Exchange::Coinbase).await;

        //gets the last bid and the last ask of binance offer book.
        let (binance_last_bid, binance_last_ask) = std::thread::spawn(move || {
            let bid = binance_order_book
                        .bids
                        .into_iter()
                        .last();

            let ask = binance_order_book
                        .asks
                        .into_iter()
                        .last();
            (bid, ask)
        })
        .join()
        .expect("Couldn't get binance bid and ask");

        //gets the last bid and the last ask of coinbase offer book.
        let (coinbase_last_bid, coinbase_last_ask) = {
            let bid = coinbase_order_book
                        .bids
                        .iter()
                        .last();

            let ask = coinbase_order_book
                        .asks
                        .iter()
                        .last();

            (bid, ask)

        };

        //updates the binance bid
        if let Some(bid) = binance_last_bid {
            self.binance_bid = bid.price;
        }
        
        //updates the coinbase bid
        if let Some(bid) = coinbase_last_bid {
            self.coinbase_bid = bid.price;
        }
    
        //updates the binance ask.
        if let Some(ask) = binance_last_ask {
             self.binance_ask = ask.price;
        }

        //updates the coinbase ask.                   
        if let Some(ask) = coinbase_last_ask {
            self.coinbase_ask = ask.price;
        }
    }

    pub async fn looks_for_opportunities(&mut self) {
        //creates a chronometer to measure the interval time for show "seeking arbitrage opportunity" message in the screen.
        let mut chronometer = Stopwatch::new();
        chronometer.start();

        loop {
            //updates prices
            self.update_prices().await;
            //gets the local time
            let local_time = Local::now().format("%c").to_string();

            //checks whether the bid price on binance is higher than the ask price on coinbase.
            if self.binance_bid > self.coinbase_ask {         
                let profit = &self.binance_bid - &self.coinbase_ask;
                println!("{}\nArbitrage opportunity found \nBuy: {} at {} on coinbase \nSell:  {} at {} on binance \nProfit: {}\n", 
                    local_time,
                    self.pair_as_str(),
                    self.coinbase_ask,
                    self.pair_as_str(),
                    self.binance_bid,
                    profit
                );

            //checks whether the bid price on binance is higher than the ask price on coinbase.
            } else if self.coinbase_bid > self.binance_ask {
                let profit = &self.coinbase_bid - &self.binance_ask;
                println!("{}\nArbitrage opportunity found \nBuy: {} at {} on binance \nSell:  {} at {} on coinbase \nProfit: {}\n", 
                    local_time,
                    self.pair_as_str(),
                    self.binance_ask,
                    self.pair_as_str(),
                    self.coinbase_bid,
                    profit
                );
            } 

            //every 7 seconds displays the message on the screen.
            if chronometer.elapsed_ms() >= 7_000 {
                println!("{}\nSeeking arbitrage opportunity... (None found)\npair: {}\nbinance bid: {}\ncoinbase ask: {}\ncoinbase bid: {}\nbinance ask: {}\n",
                    local_time,
                    self.pair_as_str(),
                    self.binance_bid,
                    self.coinbase_ask,
                    self.coinbase_bid,
                    self.binance_ask,
                );
                chronometer.stop();
                chronometer.reset();
                chronometer.start();
            }
        }
    }

    //converts enum Pair to str.
    fn pair_as_str(&self) -> &'static str {
        match &self.pair {
            Pair::BtcEur => "BTC-EUR",
            Pair::EthEur => "ETH-EUR",
            Pair::BtcGbp => "BTC-GBP",
        }
    }
}

#[test]
fn test_order_book_request_parse() {
    let binance_order_book = OrderBookRequest {market_pair: "BTCEUR".to_string()};
    assert_eq!(binance_order_book, OrderBookRequest::parse(Exchange::Binance, &Pair::BtcEur));

    let coinbase_order_book = OrderBookRequest {market_pair: "BTC-EUR".to_string()};
    assert_eq!(coinbase_order_book, OrderBookRequest::parse(Exchange::Coinbase, &Pair::BtcEur));
}

