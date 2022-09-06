pub mod exchange;
pub use exchange::*;
// use std::fs::File;
// use std::io::{self, BufRead, Write};
// use std::path::Path;
use std::collections::HashMap;
use std::fs::File as std_file;
use std::io::prelude::*;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tokio::{
    fs::File,
    io::{AsyncBufReadExt, BufReader},
};

#[tokio::main]
async fn main() -> Result<()> {
    let orders_db: OrdersDb = Arc::new(Mutex::new(HashMap::new()));
    let clients_db: ClientsDb = Arc::new(Mutex::new(HashMap::new()));
    let start = Instant::now();
    let file = File::open("./Clients.txt")
        .await
        .expect("Failed to open file clients");
    let reader = BufReader::new(file);
    let mut lines = reader.lines();
    while let Some(line) = lines.next_line().await.expect("Failed to read file") {
        let mut iter = line.split_whitespace();
        let name = match iter.next() {
            Some(value) => value,
            None => continue,
        };
        let balance = match iter.next() {
            Some(value) => match value.parse::<u32>() {
                Ok(res) => res,
                Err(_err) => {
                    return Err(ExchangeError::ProblemWithNumber);
                }
            },
            None => continue,
        };
        let mut client = Client::new(name, balance);
        let mut word: u8 = 65;
        for value in iter {
            let amount = match value.parse::<u32>() {
                Ok(res) => res,
                Err(_err) => {
                    return Err(ExchangeError::ProblemWithNumber);
                }
            };
            client.update_stock_balance(
                std::str::from_utf8(&[word]).unwrap(),
                amount,
                Operation::Buy,
            )?; // check later
            word = match word.checked_add(1) {
                Some(res) => res,
                None => {
                    return Err(ExchangeError::AddOverflow);
                }
            };
        }

        let clients_db = clients_db.clone();
        let _ = ExchangeOperation::add_client(clients_db, client).await;
    }

    let duration = start.elapsed();
    println!("clients adding amount of time {:?}", duration);
    let file = File::open("./Orders.txt")
        .await
        .expect("Failed to open file Orders");
    let reader = BufReader::new(file);
    let mut lines = reader.lines();
    while let Some(line) = lines.next_line().await.expect("Failed to read file") {
        let mut iter = line.split_whitespace();

        let name = match iter.next() {
            Some(value) => value,
            None => continue,
        };

        let operation = match iter.next() {
            Some(value) => match Operation::from_str(value) {
                Ok(value) => value,
                Err(_err) => {
                    return Err(ExchangeError::ProblemWithParsingOperation);
                }
            },
            None => continue,
        };

        let ticker = match iter.next() {
            Some(value) => value,
            None => continue,
        };

        let price = match iter.next() {
            Some(value) => match value.parse::<u32>() {
                Ok(res) => res,
                Err(_err) => {
                    return Err(ExchangeError::ProblemWithNumber);
                }
            },
            None => continue,
        };

        let amount = match iter.next() {
            Some(value) => match value.parse::<u32>() {
                Ok(res) => res,
                Err(_err) => {
                    return Err(ExchangeError::ProblemWithNumber);
                }
            },
            None => continue,
        };

        let order = Order::new(name, operation, ticker, price, amount);

        let clients_db = clients_db.clone();
        let orders_db = orders_db.clone();
        match ExchangeOperation::operate(orders_db, clients_db, order).await {
            Ok(_) => (),
            Err(er) => {
                println!("error {:?}", er);
            }
        };
    }
    let duration = start.elapsed();
    let client_balances = clients_db.lock().unwrap();
    let mut f = std_file::create("result.txt").expect("Unable to create file");
    for client in client_balances.values() {
        let mut stocks = client.get_stocks();
        stocks.sort_by_key(|k| k.0);

        let _ = f.write_fmt(format_args!(
            "{}\t{}\t{}\t{}\t{}\t{}\n",
            client.get_name(),
            client.get_balance(),
            stocks[0].1,
            stocks[1].1,
            stocks[2].1,
            stocks[3].1
        ));
    }
    println!("{:?} - working time", duration);
    Ok(())
}
