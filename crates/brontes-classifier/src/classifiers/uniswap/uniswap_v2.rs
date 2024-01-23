use alloy_primitives::{Address, U256};
use brontes_database::libmdbx::{tables::AddressToTokens, tx::CompressedLibmdbxTx};
use brontes_macros::action_impl;
use brontes_pricing::Protocol;
use brontes_types::normalized_actions::{NormalizedBurn, NormalizedMint, NormalizedSwap};
use reth_db::mdbx::RO;

action_impl!(
    Protocol::UniswapV2,
    crate::UniswapV2::swapCall,
    Swap,
    [Ignore<Sync>, Swap],
    call_data: true,
    logs: true,
    |trace_index,
    from_address: Address,
    target_address: Address,
     msg_sender: Address,
    call_data: swapCall,
    log_data: UniswapV2swapCallLogs,
    db_tx: &DB| {
        let data = log_data.Swap_field;
        let recipient = call_data.to;

        let tokens = db_tx.get::<AddressToTokens>(target_address).ok()??;
        let [token_0, token_1] = [tokens.token0, tokens.token1];

        let amount_0_in: U256 = data.amount0In;
        if amount_0_in == U256::ZERO {
            return Some(NormalizedSwap {
                pool: target_address,
                trace_index,
                from: from_address,
                recipient,
                token_in: token_1,
                token_out: token_0,
                amount_in: data.amount1In,
                amount_out: data.amount0Out,
            })
        } else {
            return Some(NormalizedSwap {
                trace_index,
                pool: target_address,
                from: from_address,
                recipient,
                token_in: token_0,
                token_out: token_1,
                amount_in: data.amount0In,
                amount_out: data.amount1Out,
            })
        }
    }
);

action_impl!(
    Protocol::UniswapV2,
    crate::UniswapV2::mintCall,
    Mint,
    [Possible<Ignore<Transfer>>, Ignore<Transfer>, Ignore<Sync>, Mint],
    logs: true,
    call_data: true,
    |trace_index,
     from_address: Address,
     target_address: Address,
     msg_sender: Address,
     call_data: mintCall,
     log_data: UniswapV2mintCallLogs,
     db_tx: &DB| {
        let log_data = log_data.Mint_field;
        let tokens = db_tx.get::<AddressToTokens>(target_address).ok()??;
        let [token_0, token_1] = [tokens.token0, tokens.token1];
        Some(NormalizedMint {
            recipient: call_data.to,
            from: from_address,
            trace_index,
            to: target_address,
            token: vec![token_0, token_1],
            amount: vec![log_data.amount0, log_data.amount1],
        })
    }
);
action_impl!(
    Protocol::UniswapV2,
    crate::UniswapV2::burnCall,
    Burn,
    [Possible<Ignore<Transfer>>, Ignore<Transfer>, Ignore<Sync>, Burn],
    call_data: true,
    logs: true,
    |trace_index,
     from_address: Address,
     target_address: Address,
     msg_sender: Address,
     call_data: burnCall,
     log_data: UniswapV2burnCallLogs,
     db_tx: &DB| {
        let log_data = log_data.Burn_field;
        let tokens = db_tx.get::<AddressToTokens>(target_address).ok()??;
        let [token_0, token_1] = [tokens.token0, tokens.token1];
        Some(NormalizedBurn {
            recipient: call_data.to,
            to: target_address,
            trace_index,
            from: from_address,
            token: vec![token_0, token_1],
            amount: vec![log_data.amount0, log_data.amount1],
        })
    }
);
