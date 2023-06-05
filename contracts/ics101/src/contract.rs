use std::ops::{Div, Mul};

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Coin, DepsMut, Env, IbcMsg, IbcTimeout, MessageInfo, Response, StdError, StdResult,
    Uint128,
};

use cw2::set_contract_version;

use crate::error::ContractError;
use crate::market::{InterchainMarketMaker, PoolStatus};
use crate::msg::{
    ExecuteMsg, InstantiateMsg, MsgCreatePoolRequest, MsgMultiAssetDepositRequest,
    MsgMultiAssetWithdrawRequest, MsgSingleAssetDepositRequest, MsgSingleAssetWithdrawRequest,
    MsgSwapRequest, SwapMsgType,
};
use crate::state::POOLS;
use crate::types::{IBCSwapPacketData, StateChange, SwapMessageType};
use crate::utils::{check_slippage, get_pool_id_with_tokens};

// Version info, for migration info
const CONTRACT_NAME: &str = "ics101-interchainswap";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
const DEFAULT_TIMEOUT_TIMESTAMP_OFFSET: u64 = 600;
const MAX_FEE_RATE: u32 = 300;
const MAXIMUM_SLIPPAGE: u64 = 10000;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> StdResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    // No setup
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::CreatePool(msg) => create_pool(deps, env, info, msg),
        ExecuteMsg::SingleAssetDeposit(msg) => single_asset_deposit(deps, env, info, msg),
        ExecuteMsg::MultiAssetDeposit(msg) => multi_asset_deposit(deps, env, info, msg),
        ExecuteMsg::SingleAssetWithdraw(msg) => singe_asset_withdraw(deps, env, info, msg),
        ExecuteMsg::MultiAssetWithdraw(msg) => multi_asset_withdraw(deps, env, info, msg),
        ExecuteMsg::Swap(msg) => swap(deps, env, info, msg),
    }
}

fn create_pool(
    _deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: MsgCreatePoolRequest,
) -> Result<Response, ContractError> {
    // validate message
    let _source_port = msg.source_port.clone();
    let source_channel = msg.source_channel.clone();

    if let Err(err) = msg.validate_basic() {
        return Err(ContractError::Std(StdError::generic_err(format!(
            "Failed to validate message: {}",
            err
        ))));
    }

    let _pool_id = get_pool_id_with_tokens(&msg.tokens);

    let pool_data = to_binary(&msg).unwrap();
    let ibc_packet_data = IBCSwapPacketData {
        r#type: SwapMessageType::CreatePool,
        data: pool_data.clone(),
        state_change: None,
    };

    let ibc_msg = IbcMsg::SendPacket {
        channel_id: source_channel.clone(),
        data: to_binary(&ibc_packet_data)?,
        timeout: IbcTimeout::from(
            env.block
                .time
                .plus_seconds(DEFAULT_TIMEOUT_TIMESTAMP_OFFSET),
        ),
    };

    let res = Response::default()
        .add_message(ibc_msg)
        .add_attribute("action", "create_pool");
    Ok(res)
}

pub fn single_asset_deposit(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: MsgSingleAssetDepositRequest,
) -> Result<Response, ContractError> {
    if let Err(err) = msg.validate_basic() {
        return Err(ContractError::Std(StdError::generic_err(format!(
            "Failed to validate message: {}",
            err
        ))));
    }

    let pool_id = msg.pool_id.clone();
    let pool = POOLS.load(deps.storage, &pool_id)?;

    // Create the interchain market maker (amm).
    let amm = InterchainMarketMaker {
        pool_id: pool_id.clone(),
        pool: pool.clone(),
        fee_rate: MAX_FEE_RATE,
    };

    // Deposit single asset to the AMM.
    let pool_token = amm
        .deposit_single_asset(&msg.token)
        .map_err(|err| StdError::generic_err(format!("Failed to deposit single asset: {}", err)))?;

    let msg_data = to_binary(&msg).unwrap();
    // Construct the IBC swap packet.
    let packet_data = IBCSwapPacketData {
        r#type: SwapMessageType::SingleDeposit,
        data: msg_data, // Use proper serialization for the `data` field.
        state_change: Some(StateChange {
            in_tokens: None,
            out_tokens: None,
            pool_tokens: Some(vec![pool_token]),
        }),
    };
    // let packet = packet_data.into_packet();

    // Send the IBC swap packet.
    let ibc_msg = IbcMsg::SendPacket {
        channel_id: pool.encounter_party_channel.clone(),
        data: to_binary(&packet_data)?,
        timeout: IbcTimeout::from(
            env.block
                .time
                .plus_seconds(DEFAULT_TIMEOUT_TIMESTAMP_OFFSET),
        ),
    };

    let res = Response::default()
        .add_message(ibc_msg)
        .add_attribute("action", "single_asset_deposit");
    Ok(res)
}

fn multi_asset_deposit(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: MsgMultiAssetDepositRequest,
) -> Result<Response, ContractError> {
    // Retrieve the interchain liquidity pool from storage
    let pool = POOLS.load(deps.storage, &msg.pool_id)?;

    // Check the pool status
    if pool.status == PoolStatus::PoolStatusInitial {
        // Check the initial deposit conditions
        for asset in vec![
            msg.local_deposit.token.clone(),
            msg.remote_deposit.token.clone(),
        ] {
            let ordered_amount = pool.clone().find_asset_by_denom(&asset.denom)?;
            if ordered_amount.balance.amount != asset.amount {
                return Err(ContractError::InvalidAmount {});
            }
        }
    } else {
        // Check the ratio of local amount and remote amount
        let ratio_of_tokens_in = Uint128::from(msg.local_deposit.token.amount)
            .mul(Uint128::from(1e18 as u64))
            .div(Uint128::from(msg.remote_deposit.token.amount));
        let local_asset_in_pool = pool
            .clone()
            .find_asset_by_denom(&msg.local_deposit.token.denom)?;
        let remote_asset_amount_in_pool = pool
            .clone()
            .find_asset_by_denom(&msg.remote_deposit.token.denom)?;
        let ratio_of_assets_in_pool = Uint128::from(local_asset_in_pool.balance.amount)
            .mul(Uint128::from(1e18 as u64))
            .div(Uint128::from(remote_asset_amount_in_pool.balance.amount));

        check_slippage(ratio_of_tokens_in, ratio_of_assets_in_pool, 10)
            .map_err(|err| StdError::generic_err(format!("Invalid Slippage: {}", err)))?;
    }

    // Create the interchain market maker
    let fee = MAX_FEE_RATE;
    // let amm = InterchainMarketMaker::new(pool.clone(), fee);
    let amm = InterchainMarketMaker {
        pool_id: pool.clone().pool_id,
        pool: pool.clone(),
        fee_rate: fee,
    };

    // Deposit the assets into the interchain market maker
    let pool_tokens = amm.deposit_multi_asset(&vec![
        msg.local_deposit.token.clone(),
        msg.remote_deposit.token.clone(),
    ])?;

    // Construct the IBC packet
    let packet_data = IBCSwapPacketData {
        r#type: SwapMessageType::MultiDeposit,
        data: to_binary(&msg)?,
        state_change: Some(StateChange {
            pool_tokens: Some(pool_tokens),
            in_tokens: None,
            out_tokens: None,
        }),
    };

    let ibc_msg = IbcMsg::SendPacket {
        channel_id: pool.clone().encounter_party_channel,
        data: to_binary(&packet_data)?,
        timeout: IbcTimeout::from(
            env.block
                .time
                .plus_seconds(DEFAULT_TIMEOUT_TIMESTAMP_OFFSET),
        ),
    };

    let res = Response::default()
        .add_message(ibc_msg)
        .add_attribute("action", "multi_asset_deposit");
    Ok(res)
}

fn singe_asset_withdraw(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: MsgSingleAssetWithdrawRequest,
) -> Result<Response, ContractError> {
    // Get liquidity pool
    let pool = POOLS.load(deps.storage, &msg.pool_coin.denom)?;

    let fee = MAX_FEE_RATE;
    let amm = InterchainMarketMaker::new(&pool, fee);

    let out = amm.single_withdraw(msg.pool_coin.clone(), &msg.denom_out)?;

    let packet = IBCSwapPacketData {
        r#type: SwapMessageType::SingleWithdraw,
        data: to_binary(&msg)?,
        state_change: Some(StateChange {
            pool_tokens: Some(vec![msg.pool_coin.clone()]),
            in_tokens: None,
            out_tokens: Some(vec![out]),
        }),
    };

    let ibc_msg = IbcMsg::SendPacket {
        channel_id: pool.encounter_party_channel,
        data: to_binary(&packet)?,
        timeout: IbcTimeout::from(
            env.block
                .time
                .plus_seconds(DEFAULT_TIMEOUT_TIMESTAMP_OFFSET),
        ),
    };

    let res = Response::default()
        .add_message(ibc_msg)
        .add_attribute("action", "singe_asset_withdraw");
    Ok(res)
}

fn multi_asset_withdraw(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: MsgMultiAssetWithdrawRequest,
) -> Result<Response, ContractError> {
    // Get liquidity pool
    let pool = POOLS.load(deps.storage, &msg.local_withdraw.pool_coin.denom)?;

    // Check pool status
    if pool.status != PoolStatus::PoolStatusReady {
        return Err(ContractError::FailedWithdraw {
            err: format!("pool not ready for swap"),
        });
    }

    let fee = MAX_FEE_RATE;
    let amm = InterchainMarketMaker::new(&pool, fee);

    let local_out = amm.multi_asset_withdraw(
        msg.local_withdraw.pool_coin.clone(),
        &msg.local_withdraw.denom_out,
    )?;
    let remote_out = amm.multi_asset_withdraw(
        msg.remote_withdraw.pool_coin.clone(),
        &msg.remote_withdraw.denom_out,
    )?;

    let packet = IBCSwapPacketData {
        r#type: SwapMessageType::MultiWithdraw,
        data: to_binary(&msg)?,
        state_change: Some(StateChange {
            pool_tokens: Some(vec![
                msg.local_withdraw.pool_coin.clone(),
                msg.remote_withdraw.pool_coin.clone(),
            ]),
            in_tokens: None,
            out_tokens: Some(vec![local_out, remote_out]),
        }),
    };

    let ibc_msg = IbcMsg::SendPacket {
        channel_id: pool.encounter_party_channel,
        data: to_binary(&packet)?,
        timeout: IbcTimeout::from(
            env.block
                .time
                .plus_seconds(DEFAULT_TIMEOUT_TIMESTAMP_OFFSET),
        ),
    };

    let res = Response::default()
        .add_message(ibc_msg)
        .add_attribute("action", "multi_asset_deposit");
    Ok(res)
}

fn swap(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: MsgSwapRequest,
) -> Result<Response, ContractError> {
    let pool_id = get_pool_id_with_tokens(&[msg.token_in.clone(), msg.token_out.clone()]);
    let pool = POOLS.load(deps.storage, &pool_id)?;

    // Check pool status
    if pool.status != PoolStatus::PoolStatusReady {
        return Err(ContractError::FailedSwap {
            err: format!("pool not ready for swap"),
        });
    }

    // Construct the IBC data packet
    let swap_data = to_binary(&msg)?;

    let fee = MAX_FEE_RATE;
    let amm = InterchainMarketMaker::new(&pool, fee);

    let token_out: Option<Coin>;
    let msg_type: Option<SwapMessageType>;

    match msg.swap_type {
        SwapMsgType::Left => {
            msg_type = Some(SwapMessageType::LeftSwap);
            token_out = Some(amm.left_swap(msg.token_in.clone(), &msg.token_out.denom)?);
        }
        SwapMsgType::Right => {
            msg_type = Some(SwapMessageType::RightSwap);
            token_out = Some(amm.right_swap(msg.token_in.clone(), msg.token_out.clone())?);
        }
    }

    let token_out = match token_out {
        Some(token) => token,
        None => {
            return Err(ContractError::FailedSwap {
                err: "token_out not found".to_string(),
            })
        }
    };

    // Slippage checking
    let factor = MAXIMUM_SLIPPAGE - msg.slippage;
    let expected = msg
        .token_out
        .amount
        .mul(Uint128::from(factor))
        .div(Uint128::from(MAXIMUM_SLIPPAGE));
    if token_out.amount.lt(&expected) {
        return Err(ContractError::FailedOnSwapReceived {
            err: format!(
                "slippage check failed! expected: {}, output: {:?}, factor: {}",
                expected, token_out, factor
            ),
        });
    }

    let packet = IBCSwapPacketData {
        r#type: msg_type.ok_or(ContractError::FailedSwap {
            err: "msg_type not found".to_string(),
        })?,
        data: swap_data,
        state_change: Some(StateChange {
            in_tokens: None,
            out_tokens: Some(vec![token_out]),
            pool_tokens: None,
        }),
    };

    let ibc_msg = IbcMsg::SendPacket {
        channel_id: pool.encounter_party_channel,
        data: to_binary(&packet)?,
        timeout: IbcTimeout::from(
            env.block
                .time
                .plus_seconds(DEFAULT_TIMEOUT_TIMESTAMP_OFFSET),
        ),
    };

    let res = Response::default()
        .add_message(ibc_msg)
        .add_attribute("action", "swap");
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};

    #[test]
    fn test_instantiate() {
        let mut deps = mock_dependencies();

        // Instantiate an empty contract
        let instantiate_msg = InstantiateMsg {};
        let info = mock_info("anyone", &[]);
        let res = instantiate(deps.as_mut(), mock_env(), info, instantiate_msg).unwrap();
        assert_eq!(0, res.messages.len());
    }
}
