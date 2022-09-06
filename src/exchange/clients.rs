use crate::{ExchangeError, Operation, Result};
use std::cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd};
use std::collections::HashMap;
// use crate::Operation;

#[derive(Debug, Clone, Default)]
pub struct Client {
    pub name: String,
    balance: u32,
    amount_of_stocks: HashMap<String, u32>,
}

impl Client {
    pub fn new(name: &str, balance: u32) -> Self {
        Client {
            name: name.to_string(),
            balance,
            amount_of_stocks: HashMap::new(),
        }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_balance(&self) -> u32 {
        self.balance
    }

    pub fn get_amount_of_stock(&self, ticker: &str) -> &u32 {
        self.amount_of_stocks.get(ticker).unwrap_or(&0)
    }

    pub fn get_stocks(&self) -> Vec<(&String, &u32)> {
        Vec::from_iter(self.amount_of_stocks.iter())
    }

    pub fn update_stock_balance(
        &mut self,
        ticker: &str,
        amount: u32,
        operation: Operation,
    ) -> Result<()> {
        match operation {
            Operation::Buy => {
                *self.amount_of_stocks.entry(ticker.to_string()).or_insert(0) += amount;
            }
            Operation::Sell => {
                // if
                let already_amount = self.amount_of_stocks.get_mut(ticker).unwrap();
                *already_amount = match already_amount.checked_sub(amount) {
                    Some(value) => value,
                    None => {
                        return Err(ExchangeError::SubtractionOverflow);
                    }
                }
            }
        }
        Ok(())
    }

    pub fn update_balance(
        &mut self,
        ticker: &str,
        amount: u32,
        operation: Operation,
        price: u32,
    ) -> Result<()> {
        let cost_of_tx = amount.checked_mul(price).unwrap();
        if (self.balance < cost_of_tx) && (operation == Operation::Buy) {
            return Err(ExchangeError::BuyerDoesntHaveEnoughMoney);
        }
        match self.update_stock_balance(ticker, amount, operation) {
            Ok(_) => (),
            Err(err) => {
                println!("{:?}", err);
                return Err(ExchangeError::SubtractionOverflow);
            }
        };
        match operation {
            Operation::Buy => {
                self.balance = self.balance.checked_sub(cost_of_tx).unwrap();
            }
            Operation::Sell => {
                self.balance = self.balance.checked_add(cost_of_tx).unwrap();
            }
        }
        Ok(())
    }
}

impl Ord for Client {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name.cmp(&other.name)
    }
}

impl PartialOrd for Client {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Client {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for Client {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_creation_and_getters() {
        let client = Client::new("C", 0);
        assert_eq!(client.get_name(), "C");
        assert_eq!(client.get_balance(), 0);
        assert_eq!(client.get_stocks().len(), 0);
    }

    #[test]
    fn test_add_stock() {
        let mut client = Client::new("C", 0);
        let _ = client.update_stock_balance("A", 10, Operation::Buy);
        assert_eq!(client.get_stocks().len(), 1);
        assert_eq!(*client.get_amount_of_stock("A"), 10);
        let _ = client.update_stock_balance("B", 20, Operation::Buy);
        assert_eq!(client.get_stocks().len(), 2);
        assert_eq!(*client.get_amount_of_stock("A"), 10);
        assert_eq!(*client.get_amount_of_stock("B"), 20);
        let _ = client.update_stock_balance("B", 30, Operation::Buy);
        assert_eq!(client.get_stocks().len(), 2);
        assert_eq!(*client.get_amount_of_stock("A"), 10);
        assert_eq!(*client.get_amount_of_stock("B"), 50);
    }

    #[test]
    fn test_sub_stock() {
        let mut client = Client::new("C", 0);
        let _ = client.update_stock_balance("A", 100, Operation::Buy);
        assert_eq!(client.get_stocks().len(), 1);
        assert_eq!(*client.get_amount_of_stock("A"), 100);
        let _ = client.update_stock_balance("A", 20, Operation::Sell);
        assert_eq!(*client.get_amount_of_stock("A"), 80);
        let _ = client.update_stock_balance("A", 20, Operation::Sell);
        assert_eq!(*client.get_amount_of_stock("A"), 60);

        let _ = client.update_stock_balance("B", 30, Operation::Buy);
        assert_eq!(client.get_stocks().len(), 2);
        assert_eq!(*client.get_amount_of_stock("A"), 60);
        assert_eq!(*client.get_amount_of_stock("B"), 30);
        let _ = client.update_stock_balance("B", 20, Operation::Sell);
        assert_eq!(client.get_stocks().len(), 2);
        assert_eq!(*client.get_amount_of_stock("A"), 60);
        assert_eq!(*client.get_amount_of_stock("B"), 10);
    }

    #[tokio::test]
    async fn test_update_balance() {
        let mut client = Client::new("C", 0);
        let _ = client.update_stock_balance("A", 100, Operation::Buy);
        assert_eq!(client.get_stocks().len(), 1);
        assert_eq!(*client.get_amount_of_stock("A"), 100);
        let _ = client.update_balance("A", 40, Operation::Sell, 10);
        assert_eq!(client.get_stocks().len(), 1);
        assert_eq!(*client.get_amount_of_stock("A"), 60);
        assert_eq!(client.get_balance(), 40 * 10);

        let _ = client.update_balance("A", 40, Operation::Sell, 10);
        assert_eq!(client.get_stocks().len(), 1);
        assert_eq!(*client.get_amount_of_stock("A"), 20);
        assert_eq!(client.get_balance(), 40 * 20);

        let _ = client.update_balance("A", 40, Operation::Buy, 10);
        assert_eq!(client.get_stocks().len(), 1);
        assert_eq!(*client.get_amount_of_stock("A"), 60);
        assert_eq!(client.get_balance(), 40 * 10);
    }
}
