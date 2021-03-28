mod arbitrage;
mod client;
mod exchange;
mod pair;

use arbitrage::Arbitrage;
use std::process::Command;
use pair::Pair;
use reader;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn clear() {
	if cfg!(windows) {
		let _ = Command::new("cmd.exe").arg("/c").arg("cls").status();
	} else {
	    let _ = Command::new("sh").arg("-c").arg("clear").status();
	}
}

#[tokio::main]
async fn main() -> Result<()> {     
    clear();

    println!("------------------------------------");
    println!("          ARBITRAGE TRADING         ");
    println!("------------------------------------\n\n");
    println!("Choose the currency pair you want to arbitrage\n");
    println!("1 --- BTC-EUR");
    println!("2 --- BTC-GBP");
    println!("3 --- ETH-EUR");
    println!();
    let pair = &reader::input("Option: ")[..];

    match pair {
        "1" => {
            clear();
            println!("Looking for arbitrage opportunities...\n");
            let mut arbitrage = Arbitrage::new(Pair::BtcEur).await;
            arbitrage.looks_for_opportunities().await;
        },
        "2" => {
            clear();
            println!("Looking for arbitrage opportunities...\n");
            let mut arbitrage = Arbitrage::new(Pair::BtcGbp).await;
            arbitrage.looks_for_opportunities().await;
        },
        "3" => {
            clear();
            println!("Looking for arbitrage opportunities...\n");
            let mut arbitrage = Arbitrage::new(Pair::EthEur).await;
            arbitrage.looks_for_opportunities().await;
        },
        _ => {
            clear();
            println!("Invalid pair");
        }
    }

    Ok(())
}