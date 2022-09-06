use std::ops::Not;
use std::str::FromStr;

#[derive(PartialEq, Clone, Copy, Debug, Eq, Hash)]
pub enum Operation {
    Buy,
    Sell,
}

impl Not for Operation {
    type Output = Self;

    fn not(self) -> Self {
        if self == Operation::Buy {
            return Operation::Sell;
        }
        Operation::Buy
    }
}

impl FromStr for Operation {
    type Err = ();

    fn from_str(input: &str) -> std::result::Result<Operation, Self::Err> {
        match input {
            "b" => Ok(Operation::Buy),
            "s" => Ok(Operation::Sell),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Order {
    client: String,
    operation: Operation,
    ticker: String,
    price: u32,
    amount: u32,
}

impl Order {
    pub fn new(client: &str, operation: Operation, ticker: &str, price: u32, amount: u32) -> Self {
        Order {
            client: client.to_string(),
            ticker: ticker.to_string(),
            price,
            amount,
            operation,
        }
    }

    pub fn get_client(&self) -> &str {
        &self.client
    }

    pub fn get_ticker(&self) -> &str {
        &self.ticker
    }

    pub fn get_price(&self) -> u32 {
        self.price
    }

    pub fn get_amount(&self) -> u32 {
        self.amount
    }

    pub fn get_operation(&self) -> Operation {
        self.operation
    }

    pub fn sub_amount(&mut self, new_amount: u32) {
        self.amount = self.amount.checked_sub(new_amount).unwrap(); // because we pick minimum, where we use it
    }

    pub fn compare_for_tx(&self, rhs: &Self) -> bool {
        (self.get_amount() == rhs.get_amount())
            && (self.get_price() == rhs.get_price())
            && (self.get_operation() != rhs.get_operation())
            && (self.get_ticker() == rhs.get_ticker())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_creation() {
        let order = Order::new("C1", Operation::Buy, "A", 10, 20);
        assert_eq!(order.get_client(), "C1");
        assert_eq!(order.get_ticker(), "A");
        assert_eq!(order.get_price(), 10);
        assert_eq!(order.get_amount(), 20);
    }

    #[test]
    fn test_compare() {
        let order = Order::new("C1", Operation::Buy, "A", 10, 20);
        assert_eq!(order.get_client(), "C1");
        assert_eq!(order.get_ticker(), "A");
        assert_eq!(order.get_price(), 10);
        assert_eq!(order.get_amount(), 20);

        let new_order = Order::new("C2", Operation::Sell, "A", 10, 20);
        assert_eq!(new_order.get_client(), "C2");
        assert_eq!(new_order.get_ticker(), "A");
        assert_eq!(new_order.get_price(), 10);
        assert_eq!(new_order.get_amount(), 20);

        assert!(order.compare_for_tx(&new_order));

        let new_order = Order::new("C2", Operation::Sell, "B", 10, 20);
        assert_eq!(new_order.get_client(), "C2");
        assert_eq!(new_order.get_ticker(), "B");
        assert_eq!(new_order.get_price(), 10);
        assert_eq!(new_order.get_amount(), 20);

        assert!(!order.compare_for_tx(&new_order));

        let new_order = Order::new("C2", Operation::Sell, "A", 20, 20);
        assert_eq!(new_order.get_client(), "C2");
        assert_eq!(new_order.get_ticker(), "A");
        assert_eq!(new_order.get_price(), 20);
        assert_eq!(new_order.get_amount(), 20);

        assert!(!order.compare_for_tx(&new_order));

        let new_order = Order::new("C2", Operation::Sell, "A", 10, 10);
        assert_eq!(new_order.get_client(), "C2");
        assert_eq!(new_order.get_ticker(), "A");
        assert_eq!(new_order.get_price(), 10);
        assert_eq!(new_order.get_amount(), 10);

        assert!(!order.compare_for_tx(&new_order));
    }
}
