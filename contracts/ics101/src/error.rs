use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Never {}

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Send some coins to create an atomic swap")]
    EmptyBalance {},

    #[error("Atomic swap not yet expired")]
    NotExpired,

    #[error("Expired atomic swap")]
    Expired,

    #[error("Atomic swap already exists")]
    AlreadyExists,

    #[error("Order already taken")]
    OrderTaken,

    #[error("Order is not for this chain")]
    InvalidChain,

    #[error("Invalid sell token")]
    InvalidSellToken,

    #[error("Order has already been taken")]
    AlreadyTakenOrder,

    #[error("Invalid taker address")]
    InvalidTakerAddress,

    #[error("Invalid maker address")]
    InvalidMakerAddress,

    #[error("Invalid sender address")]
    InvalidSender,

    #[error("Invalid status")]
    InvalidStatus,

    #[error("Got a submessage reply with unknown id: {id}")]
    UnknownReplyId { id: u64 },

    #[error("Pool is not ready for swap!")]
    NotReadyForSwap,

    #[error("Only supports channel with ibc version ics101-1, got {version}")]
    InvalidIbcVersion { version: String },

    #[error("Only supports unordered channel")]
    OnlyOrderedChannel {},

    #[error("Only accepts tokens that originate on this chain, not native tokens of remote chain")]
    NoForeignTokens {},

    #[error("Parsed port from denom ({port}) doesn't match packet")]
    FromOtherPort { port: String },

    #[error("Parsed channel from denom ({channel}) doesn't match packet")]
    FromOtherChannel { channel: String },

    #[error("Invalid denom pair")]
    InvalidDenomPair,

    #[error("Invalid decimal pair")]
    InvalidDecimalPair,

    #[error("Invalid weight pair")]
    InvalidWeightPair,

    #[error("Invalid amount")]
    InvalidAmount,

    #[error("Invalid slippage")]
    InvalidSlippage,

    #[error("Invalid pair ratio")]
    InvalidPairRatio,

    #[error("Failed to withdraw")]
    FailedWithdraw { err: String },

    #[error("Failed to swap")]
    FailedSwap { err: String },

    #[error("Failed to on swap received")]
    FailedOnSwapReceived { err: String },

    #[error("Invalid swap type")]
    InvalidSwapType,

    #[error("Invalid assets input length")]
    InvalidAssetInput,

    #[error("Error previous order is not completed")]
    ErrPreviousOrderNotCompleted,

    #[error("Error order is already completed")]
    ErrOrderAlreadyCompleted,

    #[error("Order not found")]
    ErrOrderNotFound,

    #[error("Error failed multi asset deposit")]
    ErrFailedMultiAssetDeposit
}
