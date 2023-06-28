#[cfg(not(feature = "library"))]
use anyhow::{ensure, Context};
use cosmwasm_std::{
    coin, from_binary, Binary, BankMsg, Deps, DepsMut, Env, IbcTimeout, IbcMsg, Reply, Response, SubMsg,
};
use cw_token_bridge::msg::{
    CompleteTransferResponse,
};
use std::str;
use cw_wormhole::byte_utils::ByteUtils;
use crate::state::CHAIN_TO_CHANNEL_MAP;
use crate::{
    bindings::CreateDenomResponse,
    execute::contract_addr_from_base58,
    msg::CREATE_DENOM_REPLY_ID,
};


use crate::{
    msg::GatewayIbcTokenBridgePayload,
    state::{CURRENT_TRANSFER, CW_DENOMS},
    bindings::{TokenFactoryMsg, TokenMsg},
};

pub fn handle_complete_transfer_reply(
    deps: DepsMut,
    env: Env,
    msg: Reply,
) -> Result<Response<TokenFactoryMsg>, anyhow::Error> {
    // we should only be replying on success
    ensure!(
        msg.result.is_ok(),
        "msg result is not okay, we should never get here"
    );

    let res_data_raw = cw_utils::parse_reply_execute_data(msg)
        .context("failed to parse protobuf reply response_data")?
        .data
        .context("no data in the response, we should never get here")?;
    let res_data: CompleteTransferResponse =
        from_binary(&res_data_raw).context("failed to deserialize response data")?;
    let contract_addr = res_data
        .contract
        .context("no contract in response, we should never get here")?;

    // load interim state
    let transfer_info = CURRENT_TRANSFER
        .load(deps.storage)
        .context("failed to load current transfer from storage")?;

    // delete interim state
    CURRENT_TRANSFER.remove(deps.storage);

    // deserialize payload into the type we expect
    let payload: GatewayIbcTokenBridgePayload = serde_json_wasm::from_slice(&transfer_info.payload)
        .context("failed to deserialize transfer payload")?;
 
    match payload {
        GatewayIbcTokenBridgePayload::Simple { chain, recipient, fee: _, nonce: _ } => {
            let recipient_decoded = String::from_utf8(recipient.to_vec()).context(format!(
                "failed to convert {} to utf8 string",
                recipient.to_string()
            ))?;
            convert_cw20_to_bank_and_send(
                deps,
                env,
                recipient_decoded,
                res_data.amount.into(),
                contract_addr,
                chain,
                None,
            )
        },
        GatewayIbcTokenBridgePayload::ContractControlled { chain, contract, payload, nonce: _ } => {
            let contract_decoded = String::from_utf8(contract.to_vec()).context(format!(
                "failed to convert {} to utf8 string",
                contract.to_string()
            ))?;
            convert_cw20_to_bank_and_send(
                deps,
                env,
                contract_decoded,
                res_data.amount.into(),
                contract_addr,
                chain,
                Some(payload),
            )
        }
    }
}
pub fn convert_cw20_to_bank_and_send(
    deps: DepsMut,
    env: Env,
    recipient: String,
    amount: u128,
    contract_addr: String,
    chain_id: u16,
    payload: Option<Binary>,
) -> Result<Response<TokenFactoryMsg>, anyhow::Error> {
    // check the recipient and contract addresses are valid
    // recipient will have a different bech32 prefix and fail
    //deps.api
    //    .addr_validate(&recipient)
    //    .context(format!("invalid recipient address {}", recipient))?;

    deps.api
        .addr_validate(&contract_addr)
        .context(format!("invalid contract address {}", contract_addr))?;

    // convert contract address into base64
    let subdenom = contract_addr_to_base58(deps.as_ref(), contract_addr.clone())?;
    // format the token factory denom
    let tokenfactory_denom = "factory/".to_string()
        + env.contract.address.to_string().as_ref()
        + "/"
        + subdenom.as_ref();

    let mut response: Response<TokenFactoryMsg> = Response::new();

    // check contract storage see if we've created a denom for this cw20 token yet
    // if we haven't created the denom, then create the denom
    // info.sender contains the cw20 contract address
    if !CW_DENOMS.has(deps.storage, contract_addr.clone()) {
        // call into token factory to create the denom
        let create_denom = SubMsg::reply_on_success(
            TokenMsg::CreateDenom { 
                subdenom: subdenom.clone(), 
                metadata: None,
            },
            CREATE_DENOM_REPLY_ID,
        );
        response = response.add_submessage(create_denom);

        // add the contract_addr => tokenfactory denom to storage
        CW_DENOMS
            .save(deps.storage, contract_addr, &tokenfactory_denom)
            .context("failed to save contract_addr => tokenfactory denom to storage")?;
    }

    // add calls to mint and send bank tokens
    response = response.add_message(TokenMsg::MintTokens {
        denom: tokenfactory_denom.clone(),
        amount: amount.clone(),
        mint_to_address: env.contract.address.to_string(),
    });

    // amount of tokenfactory coins to ibc transfer
    let amount = coin(amount, tokenfactory_denom);

    let channel = CHAIN_TO_CHANNEL_MAP
        .load(deps.storage, chain_id)
        .context("chain id does not have an allowed channel")?;

    let channel_entry = match payload {
        Some(payload) => {
            let payload_decoded = String::from_utf8(payload.to_vec()).context(format!(
                "failed to convert {} to utf8 string",
                payload.to_string()
            ))?;
            channel + "," + &payload_decoded
        },
        None => channel
    };

    response = response.add_message( IbcMsg::Transfer { 
        channel_id: channel_entry, 
        to_address: recipient, 
        amount: amount, 
        timeout: IbcTimeout::with_timestamp(env.block.time.plus_minutes(2)),
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

pub fn handle_create_denom_reply(
    _deps: DepsMut,
    _env: Env,
    msg: Reply,
) -> Result<Response<TokenFactoryMsg>, anyhow::Error> {
    // we should only be replying on success
    ensure!(
        msg.result.is_ok(),
        "msg result is not okay, we should never get here"
    );
    
    // extract the contract address from the create denom response
    // if the token is not a factory token created by this contract, return error
    // let parsed_denom = new_token_denom.split("/").collect::<Vec<_>>();
    // ensure!(
    //    parsed_denom.len() == 3
    //        && parsed_denom[0] == "factory"
    //        && parsed_denom[1] == env.contract.address.to_string(),
    //    "coin is not from the token factory"
    //);

    // decode subdenom from base64 => encode as cosmos addr to get contract addr
    //let cw20_contract_addr = contract_addr_from_base58(deps.as_ref(), parsed_denom[2])?;

    // validate that the contract does indeed match the stored denom we have for it
    //let _stored_denom = CW_DENOMS
    //    .load(deps.storage, cw20_contract_addr)
    //    .context(
    //        "a corresponding denom for the extracted contract addr is not contained in storage",
    //    )?;

    Ok(Response::new())
}