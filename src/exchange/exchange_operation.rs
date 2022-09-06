use crate::{Client, ExchangeError, Operation, Order, Result};
use std::cmp::min;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub type ClientsDb = Arc<Mutex<HashMap<ClientsName, Client>>>;
pub type OrdersDb = Arc<Mutex<HashMap<Ticker, HashMap<Operation, HashMap<Price, Vec<Order>>>>>>;
type Price = u32;
type ClientsName = String;
type Ticker = String;

#[derive(Debug)]
pub struct ExchangeOperation {}

impl ExchangeOperation {
    pub async fn add_client(clients_base: ClientsDb, client: Client) -> Result<()> {
        let mut clients_base = clients_base.lock().unwrap();
        if clients_base.contains_key(client.get_name()) {
            return Err(ExchangeError::UserAlreadyRegistered);
        }
        clients_base.insert(client.get_name().to_string(), client);
        Ok(())
    }

    pub async fn operate(
        orders_base: OrdersDb,
        clients_base: ClientsDb,
        new_order: Order,
    ) -> Result<()> {
        let mut orders_base = orders_base.lock().unwrap();
        let mut clients_base = clients_base.lock().unwrap();
        if !clients_base.contains_key(new_order.get_client()) {
            return Err(ExchangeError::UnknownUser);
        }
        if (clients_base
            .get(new_order.get_client())
            .unwrap()
            .get_amount_of_stock(new_order.get_ticker())
            < &new_order.get_amount())
            && (new_order.get_operation() == Operation::Sell)
        {
            return Ok(());
        }

        let mut mut_new_order = Order::new(
            new_order.get_client(),
            new_order.get_operation(),
            new_order.get_ticker(),
            new_order.get_price(),
            new_order.get_amount(),
        );
        let needed_amount = mut_new_order.get_price() * mut_new_order.get_amount();
        let order_operation = mut_new_order.get_operation();
        let order_price = mut_new_order.get_price();

        if (needed_amount
            > clients_base
                .get(mut_new_order.get_client())
                .unwrap()
                .get_balance())
            && (mut_new_order.get_operation() == Operation::Buy)
        {
            return Ok(());
        }

        let mut flag_for_add = true;
        if let Some(operation_to_price_to_orders) = orders_base.get_mut(mut_new_order.get_ticker())
        {
            flag_for_add = false;
            let mut flag_for_price_to_orders = true;
            while let (Some(price_to_orders), amount) = (
                operation_to_price_to_orders.get_mut(&!order_operation),
                mut_new_order.get_amount(),
            ) {
                if amount == 0 {
                    break;
                }
                flag_for_price_to_orders = false;
                let (price, price_is_acceptable, flag_for_add_update) =
                    Self::calculate_difference_of_path(
                        order_operation,
                        price_to_orders.keys(),
                        order_price,
                    );
                if flag_for_add_update {
                    flag_for_add = true;
                    break;
                }
                if price_is_acceptable {
                    let orders = price_to_orders.get_mut(&price).unwrap();
                    while !orders.is_empty() && (mut_new_order.get_amount() > 0) {
                        let orders_len = orders.len();
                        let order = orders.first_mut().unwrap();
                        let (buyer, seller): (&mut Client, &mut Client) = match order_operation {
                            Operation::Sell => Self::get_mut_pair(
                                &mut clients_base,
                                &order.get_client().to_string(),
                                &mut_new_order.get_client().to_string(),
                            ),
                            Operation::Buy => Self::get_mut_pair(
                                &mut clients_base,
                                &mut_new_order.get_client().to_string(),
                                &order.get_client().to_string(),
                            ),
                        };

                        if buyer.get_balance() < order.get_amount() * order.get_price() {
                            orders.remove(0);
                            continue;
                        }

                        let amount = min(mut_new_order.get_amount(), order.get_amount());

                        let buyer_res = buyer.update_balance(
                            mut_new_order.get_ticker(),
                            amount,
                            Operation::Buy,
                            price,
                        );
                        let seller_res = seller.update_balance(
                            mut_new_order.get_ticker(),
                            amount,
                            Operation::Sell,
                            price,
                        );
                        match buyer_res {
                            Ok(_) => (),
                            Err(err) => {
                                return Err(err);
                            }
                        };
                        match seller_res {
                            Ok(_) => (),
                            Err(err) => {
                                return Err(err);
                            }
                        };
                        mut_new_order.sub_amount(amount);
                        order.sub_amount(amount);
                        if order.get_amount() == 0 {
                            orders.remove(0);
                            assert_eq!(orders.len(), orders_len - 1);
                        }
                    }
                    if orders.is_empty() {
                        price_to_orders.remove(&price).unwrap();
                    }
                }
                if (!price_is_acceptable) && (mut_new_order.get_amount() > 0) {
                    if let Some(price_to_orders) =
                        operation_to_price_to_orders.get_mut(&order_operation)
                    {
                        if let Some(orders) = price_to_orders.get_mut(&order_price) {
                            orders.push(Order::new(
                                mut_new_order.get_client(),
                                mut_new_order.get_operation(),
                                mut_new_order.get_ticker(),
                                mut_new_order.get_price(),
                                mut_new_order.get_amount(),
                            ));
                        } else {
                            price_to_orders.insert(
                                order_price,
                                vec![Order::new(
                                    mut_new_order.get_client(),
                                    mut_new_order.get_operation(),
                                    mut_new_order.get_ticker(),
                                    mut_new_order.get_price(),
                                    mut_new_order.get_amount(),
                                )],
                            );
                        }
                    } else {
                        let mut price_to_orders: HashMap<u32, Vec<Order>> = HashMap::new();
                        price_to_orders.insert(
                            order_price,
                            vec![Order::new(
                                mut_new_order.get_client(),
                                mut_new_order.get_operation(),
                                mut_new_order.get_ticker(),
                                mut_new_order.get_price(),
                                mut_new_order.get_amount(),
                            )],
                        );
                        operation_to_price_to_orders.insert(order_operation, price_to_orders);
                    }
                    break;
                }
            }
            if flag_for_price_to_orders {
                if let Some(price_to_orders) =
                    operation_to_price_to_orders.get_mut(&order_operation)
                {
                    if let Some(orders) = price_to_orders.get_mut(&order_price) {
                        orders.push(mut_new_order);
                    } else {
                        price_to_orders.insert(new_order.get_price(), vec![mut_new_order]);
                    }
                } else {
                    let mut price_to_orders: HashMap<u32, Vec<Order>> = HashMap::new();
                    price_to_orders.insert(
                        new_order.get_price(),
                        vec![Order::new(
                            mut_new_order.get_client(),
                            mut_new_order.get_operation(),
                            mut_new_order.get_ticker(),
                            mut_new_order.get_price(),
                            mut_new_order.get_amount(),
                        )],
                    );
                    operation_to_price_to_orders.insert(order_operation, price_to_orders);
                }
            }
        }
        if flag_for_add {
            let ticker = new_order.get_ticker().to_string();
            let mut operation_to_price_to_orders: HashMap<Operation, HashMap<u32, Vec<Order>>> =
                HashMap::new();
            let mut price_to_orders: HashMap<u32, Vec<Order>> = HashMap::new();
            price_to_orders.insert(new_order.get_price(), vec![new_order]);
            operation_to_price_to_orders.insert(order_operation, price_to_orders);
            orders_base.insert(ticker, operation_to_price_to_orders);
        }
        Ok(())
    }

    fn get_mut_pair<'a, K, V>(conns: &'a mut HashMap<K, V>, a: &K, b: &K) -> (&'a mut V, &'a mut V)
    where
        K: Eq + std::hash::Hash,
    {
        unsafe {
            let a = conns.get_mut(a).unwrap() as *mut _;
            let b = conns.get_mut(b).unwrap() as *mut _;
            // assert_ne!(a, b, "The two keys must not resolve to the same value");
            (&mut *a, &mut *b)
        }
    }

    fn calculate_difference_of_path<'a>(
        order_operation: Operation,
        keys: impl Iterator<Item = &'a u32>,
        order_price: u32,
    ) -> (u32, bool, bool) {
        if order_operation == Operation::Sell {
            let max_buying_price = match keys.max() {
                None => {
                    return (0, false, true);
                }
                Some(val) => *val,
            };
            return (max_buying_price, max_buying_price >= order_price, false);
        }
        let min_selling_price = match keys.min() {
            None => {
                return (0, false, true);
            }
            Some(val) => *val,
        };
        (min_selling_price, min_selling_price <= order_price, false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_addition_of_clients() {
        let clients_db: Arc<Mutex<HashMap<ClientsName, Client>>> =
            Arc::new(Mutex::new(HashMap::new()));
        let client1 = Client::new("A", 5);
        let client2 = Client::new("B", 4);
        match ExchangeOperation::add_client(clients_db.clone(), client1.clone()).await {
            Ok(_) => (),
            Err(er) => {
                println!("error with adding {}", er);
                ()
            }
        }
        assert_eq!(clients_db.clone().lock().unwrap().len(), 1);
        match ExchangeOperation::add_client(clients_db.clone(), client2.clone()).await {
            Ok(_) => (),
            Err(er) => {
                println!("error with adding {}", er);
                ()
            }
        }
        assert_eq!(clients_db.clone().lock().unwrap().len(), 2);

        match ExchangeOperation::add_client(clients_db.clone(), client2.clone()).await {
            Ok(_) => {
                println!("there should be error");
                ()
            }
            Err(_) => (),
        }

        assert_eq!(clients_db.clone().lock().unwrap().len(), 2);
    }

    #[tokio::test]
    async fn test_exchange() {
        let orders_db: Arc<Mutex<HashMap<Ticker, HashMap<Operation, HashMap<Price, Vec<Order>>>>>> =
            Arc::new(Mutex::new(HashMap::new()));
        let clients_db: Arc<Mutex<HashMap<ClientsName, Client>>> =
            Arc::new(Mutex::new(HashMap::new()));
        let mut client1 = Client::new("A", 1000);
        let mut client2 = Client::new("B", 1000);
        let _ = client2.update_stock_balance("C1", 10, Operation::Buy);
        let _ = client1.update_stock_balance("C1", 10, Operation::Buy);
        match ExchangeOperation::add_client(clients_db.clone(), client1.clone()).await {
            Ok(_) => (),
            Err(er) => {
                println!("error with adding client {}", er);
                ()
            }
        }
        match ExchangeOperation::add_client(clients_db.clone(), client2.clone()).await {
            Ok(_) => (),
            Err(er) => {
                println!("error with adding client {}", er);
                ()
            }
        }
        let order1 = Order::new("B", Operation::Buy, "C1", 10, 10);
        match ExchangeOperation::operate(orders_db.clone(), clients_db.clone(), order1.clone())
            .await
        {
            Ok(_) => (),
            Err(er) => {
                println!("error with operating order1 {}", er);
                ()
            }
        }
        // println!("{:?}", exchange.get_open_orders());
        assert_eq!(
            orders_db
                .lock()
                .unwrap()
                .values()
                .flat_map(HashMap::values)
                .flat_map(HashMap::values)
                .flatten()
                .count(),
            1
        );
        assert_eq!(clients_db.lock().unwrap().len(), 2);

        let order2 = Order::new("A", Operation::Sell, "C1", 10, 10);
        match ExchangeOperation::operate(orders_db.clone(), clients_db.clone(), order2.clone())
            .await
        {
            Ok(_) => (),
            Err(er) => {
                println!("error with operating order2 {}", er);
                ()
            }
        }
        assert_eq!(
            orders_db
                .lock()
                .unwrap()
                .values()
                .flat_map(HashMap::values)
                .flat_map(HashMap::values)
                .flatten()
                .count(),
            0
        );
        assert_eq!(clients_db.lock().unwrap().len(), 2);

        let clients_db1 = clients_db.lock().unwrap();
        let client1_new = clients_db1.get("A").unwrap();

        assert_eq!(client1_new.get_balance(), 1100);
        assert_eq!(client1_new.get_amount_of_stock("C1"), &0);

        let client2_new = clients_db1.get("B").unwrap();

        assert_eq!(client2_new.get_balance(), 900);
        assert_eq!(client2_new.get_amount_of_stock("C1"), &20);
    }

    #[tokio::test]
    async fn test_no_exchange() {
        let orders_db: Arc<Mutex<HashMap<Ticker, HashMap<Operation, HashMap<Price, Vec<Order>>>>>> =
            Arc::new(Mutex::new(HashMap::new()));
        let clients_db: Arc<Mutex<HashMap<ClientsName, Client>>> =
            Arc::new(Mutex::new(HashMap::new()));
        let mut client1 = Client::new("A", 1000);
        let mut client2 = Client::new("B", 1000);
        let _ = client2.update_stock_balance("C2", 10, Operation::Buy);
        let _ = client1.update_stock_balance("C1", 10, Operation::Buy);
        match ExchangeOperation::add_client(clients_db.clone(), client1.clone()).await {
            Ok(_) => (),
            Err(er) => {
                println!("error with adding client {}", er);
                ()
            }
        }
        match ExchangeOperation::add_client(clients_db.clone(), client2.clone()).await {
            Ok(_) => (),
            Err(er) => {
                println!("error with adding client {}", er);
                ()
            }
        }
        let order1 = Order::new("B", Operation::Buy, "C2", 10, 10);
        match ExchangeOperation::operate(orders_db.clone(), clients_db.clone(), order1.clone())
            .await
        {
            Ok(_) => (),
            Err(er) => {
                println!("error with operating order1 {}", er);
                ()
            }
        }
        // println!("{:?}", exchange.get_open_orders());
        assert_eq!(
            orders_db
                .lock()
                .unwrap()
                .values()
                .flat_map(HashMap::values)
                .flat_map(HashMap::values)
                .flatten()
                .count(),
            1
        );
        assert_eq!(clients_db.lock().unwrap().len(), 2);

        let order2 = Order::new("A", Operation::Sell, "C1", 10, 10);
        match ExchangeOperation::operate(orders_db.clone(), clients_db.clone(), order2.clone())
            .await
        {
            Ok(_) => (),
            Err(er) => {
                println!("error with operating order2 {}", er);
                ()
            }
        }
        println!("{:?}", orders_db.lock().unwrap());
        assert_eq!(
            orders_db
                .lock()
                .unwrap()
                .values()
                .flat_map(HashMap::values)
                .flat_map(HashMap::values)
                .flatten()
                .count(),
            2
        );
        assert_eq!(clients_db.lock().unwrap().len(), 2);

        let order3 = Order::new("B", Operation::Sell, "C2", 11, 9);
        match ExchangeOperation::operate(orders_db.clone(), clients_db.clone(), order3.clone())
            .await
        {
            Ok(_) => (),
            Err(er) => {
                println!("error with operating order3 {}", er);
                ()
            }
        }
        assert_eq!(
            orders_db
                .lock()
                .unwrap()
                .values()
                .flat_map(HashMap::values)
                .flat_map(HashMap::values)
                .flatten()
                .count(),
            3
        );
        assert_eq!(clients_db.lock().unwrap().len(), 2);

        let order4 = Order::new("A", Operation::Buy, "C1", 9, 11);
        match ExchangeOperation::operate(orders_db.clone(), clients_db.clone(), order4.clone())
            .await
        {
            Ok(_) => (),
            Err(er) => {
                println!("error with operating order4 {}", er);
                ()
            }
        }
        assert_eq!(
            orders_db
                .lock()
                .unwrap()
                .values()
                .flat_map(HashMap::values)
                .flat_map(HashMap::values)
                .flatten()
                .count(),
            4
        );
        assert_eq!(clients_db.lock().unwrap().len(), 2);
    }

    #[tokio::test]
    async fn test_buy1() {
        let orders_db: Arc<Mutex<HashMap<Ticker, HashMap<Operation, HashMap<Price, Vec<Order>>>>>> =
            Arc::new(Mutex::new(HashMap::new()));
        let clients_db: Arc<Mutex<HashMap<ClientsName, Client>>> =
            Arc::new(Mutex::new(HashMap::new()));
        let client1 = Client::new("A", 10000);
        let mut client2 = Client::new("B", 10000);
        let _ = client2.update_stock_balance("C1", 100, Operation::Buy);
        match ExchangeOperation::add_client(clients_db.clone(), client1.clone()).await {
            Ok(_) => (),
            Err(er) => {
                println!("error with adding client {}", er);
                ()
            }
        }
        match ExchangeOperation::add_client(clients_db.clone(), client2.clone()).await {
            Ok(_) => (),
            Err(er) => {
                println!("error with adding client {}", er);
                ()
            }
        }

        let order1 = Order::new("B", Operation::Sell, "C1", 10, 50);
        match ExchangeOperation::operate(orders_db.clone(), clients_db.clone(), order1.clone())
            .await
        {
            Ok(_) => (),
            Err(er) => {
                println!("error with operating order1 {}", er);
                ()
            }
        }
        assert_eq!(
            orders_db
                .lock()
                .unwrap()
                .values()
                .flat_map(HashMap::values)
                .flat_map(HashMap::values)
                .flatten()
                .count(),
            1
        );
        assert_eq!(clients_db.lock().unwrap().len(), 2);

        let order2 = Order::new("B", Operation::Sell, "C1", 15, 50);
        match ExchangeOperation::operate(orders_db.clone(), clients_db.clone(), order2.clone())
            .await
        {
            Ok(_) => (),
            Err(er) => {
                println!("error with operating order2 {}", er);
                ()
            }
        }
        assert_eq!(
            orders_db
                .lock()
                .unwrap()
                .values()
                .flat_map(HashMap::values)
                .flat_map(HashMap::values)
                .flatten()
                .count(),
            2
        );
        assert_eq!(clients_db.lock().unwrap().len(), 2);

        let order3 = Order::new("A", Operation::Buy, "C1", 20, 75);
        match ExchangeOperation::operate(orders_db.clone(), clients_db.clone(), order3.clone())
            .await
        {
            Ok(_) => (),
            Err(er) => {
                println!("error with operating order3 {}", er);
                ()
            }
        }

        let clients_db1 = clients_db.lock().unwrap();
        let client_a = clients_db1.get("A").unwrap();
        assert_eq!(client_a.get_balance(), 10000 - (10 * 50 + 15 * 25));

        let client_b = clients_db1.get("B").unwrap();
        assert_eq!(client_b.get_balance(), 10000 + (10 * 50 + 15 * 25));

        assert_eq!(
            orders_db
                .lock()
                .unwrap()
                .values()
                .flat_map(HashMap::values)
                .flat_map(HashMap::values)
                .flatten()
                .count(),
            1
        );
        assert_eq!(clients_db1.len(), 2);
    }
}
