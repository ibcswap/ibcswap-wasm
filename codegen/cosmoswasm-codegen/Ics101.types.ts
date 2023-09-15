/**
* This file was automatically generated by @cosmwasm/ts-codegen@0.35.3.
* DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
* and run the @cosmwasm/ts-codegen generate command to regenerate this file.
*/

export type ExecuteMsg = {
  MakePool: MsgMakePoolRequest;
} | {
  TakePool: MsgTakePoolRequest;
} | {
  CancelPool: MsgCancelPoolRequest;
} | {
  SingleAssetDeposit: MsgSingleAssetDepositRequest;
} | {
  MakeMultiAssetDeposit: MsgMakeMultiAssetDepositRequest;
} | {
  CancelMultiAssetDeposit: MsgCancelMultiAssetDepositRequest;
} | {
  TakeMultiAssetDeposit: MsgTakeMultiAssetDepositRequest;
} | {
  MultiAssetWithdraw: MsgMultiAssetWithdrawRequest;
} | {
  Swap: MsgSwapRequest;
} | {
  RemovePool: MsgRemovePool;
};
export type Uint128 = string;
export type PoolSide = "SOURCE" | "DESTINATION";
export type SwapMsgType = "LEFT" | "RIGHT";
export interface MsgMakePoolRequest {
  counterpartyChannel: string;
  counterpartyCreator: string;
  creator: string;
  destinationChainId: string;
  liquidity: PoolAsset[];
  sourceChainId: string;
  sourceChannel: string;
  sourcePort: string;
  swapFee: number;
  timeoutHeight: number;
  timeoutTimestamp: number;
  [k: string]: unknown;
}
export interface PoolAsset {
  balance: Coin;
  decimal: number;
  side: PoolSide;
  weight: number;
  [k: string]: unknown;
}
export interface Coin {
  amount: Uint128;
  denom: string;
  [k: string]: unknown;
}
export interface MsgTakePoolRequest {
  counterCreator: string;
  creator: string;
  poolId: string;
  timeoutHeight: number;
  timeoutTimestamp: number;
  [k: string]: unknown;
}
export interface MsgCancelPoolRequest {
  poolId: string;
  timeoutHeight: number;
  timeoutTimestamp: number;
  [k: string]: unknown;
}
export interface MsgSingleAssetDepositRequest {
  poolId: string;
  sender: string;
  timeoutHeight: number;
  timeoutTimestamp: number;
  token: Coin;
  [k: string]: unknown;
}
export interface MsgMakeMultiAssetDepositRequest {
  chainId: string;
  deposits: DepositAsset[];
  poolId: string;
  timeoutHeight: number;
  timeoutTimestamp: number;
  [k: string]: unknown;
}
export interface DepositAsset {
  balance: Coin;
  sender: string;
  [k: string]: unknown;
}
export interface MsgCancelMultiAssetDepositRequest {
  orderId: string;
  poolId: string;
  sender: string;
  timeoutHeight: number;
  timeoutTimestamp: number;
  [k: string]: unknown;
}
export interface MsgTakeMultiAssetDepositRequest {
  orderId: string;
  poolId: string;
  sender: string;
  timeoutHeight: number;
  timeoutTimestamp: number;
  [k: string]: unknown;
}
export interface MsgMultiAssetWithdrawRequest {
  counterpartyReceiver: string;
  poolId: string;
  poolToken: Coin;
  receiver: string;
  timeoutHeight: number;
  timeoutTimestamp: number;
  [k: string]: unknown;
}
export interface MsgSwapRequest {
  poolId: string;
  recipient: string;
  sender: string;
  slippage: number;
  swapType: SwapMsgType;
  timeoutHeight: number;
  timeoutTimestamp: number;
  tokenIn: Coin;
  tokenOut: Coin;
  [k: string]: unknown;
}
export interface MsgRemovePool {
  poolId: string;
  [k: string]: unknown;
}
export interface InstantiateMsg {
  token_code_id: number;
  [k: string]: unknown;
}
export type QueryMsg = {
  OrderList: {
    limit?: number | null;
    start_after?: string | null;
    [k: string]: unknown;
  };
} | {
  Order: {
    order_id: string;
    pool_id: string;
    [k: string]: unknown;
  };
} | {
  Config: {
    [k: string]: unknown;
  };
} | {
  PoolTokenList: {
    limit?: number | null;
    start_after?: string | null;
    [k: string]: unknown;
  };
} | {
  PoolAddressByToken: {
    pool_id: string;
    [k: string]: unknown;
  };
} | {
  InterchainPool: {
    pool_id: string;
    [k: string]: unknown;
  };
} | {
  InterchainPoolList: {
    limit?: number | null;
    start_after?: string | null;
    [k: string]: unknown;
  };
} | {
  LeftSwap: {
    pool_id: string;
    token_in: Coin;
    token_out: Coin;
    [k: string]: unknown;
  };
} | {
  RightSwap: {
    pool_id: string;
    token_in: Coin;
    token_out: Coin;
    [k: string]: unknown;
  };
} | {
  QueryActiveOrders: {
    destination_taker: string;
    pool_id: string;
    source_maker: string;
    [k: string]: unknown;
  };
} | {
  Rate: {
    amount: Uint128;
    pool_id: string;
    [k: string]: unknown;
  };
};