use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Order not found")]
    OrderNotFound,

    #[error("Insufficient funds")]
    InsufficientFunds,

    #[error("User not registered")]
    UserNotRegistered,

    #[error("Item not available")]
    ItemNotAvailable,

    #[error("Incorrect payment amount")]
    IncorrectPayment,

    #[error("Already Processed")]
    OrderAlreadyProcessed,

    #[error("Not Accepted")]
    OrderNotAccepted,

    #[error("Order in delivery")]
    OrderInDelivery,

    #[error("Order not in delivery")]
    OrderNotInDelivery,

    #[error("No rider assigned")]
    NoRiderAssigned,

    #[error("Order not completed")]
    OrderNotCompleted,

    #[error("Funds already released")]
    FundsAlreadyReleased,

    #[error("Invalid fee percentage")]
    InvalidFeePercentage {},

    #[error("Invalid order amount")]
    InvalidOrderAmount {},

    #[error("Overflow occurred")]
    Overflow {},

    #[error("Insufficient escrow amount")]
    InsufficientEscrowAmount {},

    #[error("Escrow balance not loaded")]
    EscrowBalanceNotLoaded {},

    #[error("Delivery not confirmed")]
    DeliveryNotConfirmed {},

    #[error("Insufficient escrow balance")]
    InsufficientEscrowBalance {},

    #[error("Incorrect payment: {reason}")]
    IncorrectPayments { reason: String },

    #[error("Empty order")]
    EmptyOrder {},

    #[error("Item not found")]
    ItemNotFound {},

    #[error("MenuItem not found")]
    MenuItemNotFound,
}
pub type ContractResult<T> = Result<T, ContractError>;
