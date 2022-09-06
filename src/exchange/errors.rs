use thiserror::Error;
pub type Result<T> = std::result::Result<T, ExchangeError>;

#[derive(Error, Debug, Eq, PartialEq)]
pub enum ExchangeError {
    #[error("The provided client with that username already registered")]
    UserAlreadyRegistered,

    #[error("No value in hashmap")]
    NoValueInHashMap,

    #[error("The buyer does not have enough money")]
    BuyerDoesntHaveEnoughMoney,

    #[error("The seller does not have enough stock")]
    NotEnoughStocks,

    #[error("Problem with parsing of input numbers")]
    ProblemWithNumber,

    #[error("Problem with addition overflow")]
    AddOverflow,

    #[error("Problem with parsing operation from str")]
    ProblemWithParsingOperation,

    #[error("Unknown user in operation")]
    UnknownUser,

    #[error("Problem with subtraction overflow")]
    SubtractionOverflow,
}
