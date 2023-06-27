#[cfg(not(feature = "library"))]
use anyhow::{Context};
use cosmwasm_std::{
    Deps, DepsMut, Env, coin,
    MessageInfo, Response, SubMsg, IbcMsg, IbcTimeout,
};

use crate::{
    bindings::{TokenFactoryMsg, TokenMsg},
};

pub fn create_token_and_send(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    contract: String,
    amount: u128,
    payload: String,
) -> Result<Response<TokenFactoryMsg>, anyhow::Error> {

    let this_contract_addr = env.contract.address.to_string();

    let subdenom = contract_addr_to_base58(deps.as_ref(), env.contract.address.into_string())?;
    let tokenfactory_denom = "factory/".to_string()
        + this_contract_addr.as_ref()
        + "/"
        + subdenom.as_ref();
    
    let mut response: Response<TokenFactoryMsg> = Response::new();
    let create_denom = SubMsg::reply_on_success(
        TokenMsg::CreateDenom { 
            subdenom: subdenom.clone(), 
            metadata: None,
        },
        1,
    );
    response = response.add_submessage(create_denom);
    
    response = response.add_message(TokenMsg::MintTokens {
        denom: tokenfactory_denom.clone(),
        amount: amount.clone(),
        mint_to_address: this_contract_addr,
    });

    let channel = "channel-0".to_string();
    let channel_payload = channel + "," + &payload;
    
    let amount = coin(amount, tokenfactory_denom);
    response = response.add_message( IbcMsg::Transfer { 
        channel_id: channel_payload, 
        to_address: contract, 
        amount: amount, 
        timeout: IbcTimeout::with_timestamp(env.block.time.plus_days(1)),
    });

    Ok(response)
}

// Base58 allows the subdenom to be a maximum of 44 bytes (max subdenom length) for up to a 32 byte address
fn contract_addr_to_base58(deps: Deps, contract_addr: String) -> Result<String, anyhow::Error> {
    // convert the contract address into bytes
    let contract_addr_bytes = deps.api.addr_canonicalize(&contract_addr).context(format!(
        "could not canonicalize contract address {}",
        contract_addr
    ))?;
    let base_58_addr = bs58::encode(contract_addr_bytes.as_slice()).into_string();
    Ok(base_58_addr)
}