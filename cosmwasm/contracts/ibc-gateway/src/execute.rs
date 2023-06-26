#[cfg(not(feature = "library"))]
use anyhow::{bail, ensure, Context};
use cosmwasm_std::{
    coin, from_binary, to_binary, BankMsg, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, Empty, Event,
    MessageInfo, QueryRequest, Reply, Response, SubMsg, StdResult, StdError, Uint128, WasmMsg, WasmQuery,
};
use cw20::Cw20ExecuteMsg;
use cw_token_bridge::msg::{
    Asset, AssetInfo, CompleteTransferResponse, ExecuteMsg as TokenBridgeExecuteMsg,
    QueryMsg as TokenBridgeQueryMsg, TransferInfoResponse,
};
use cw_wormhole::{
    byte_utils::ByteUtils,
    msg::QueryMsg as WormholeQueryMsg,
    state::{ParsedVAA},
};

use cw20_wrapped_2::msg::ExecuteMsg as Cw20WrappedExecuteMsg;
use wormhole_sdk::{
    vaa::{Body, Header},
    ibc_shim::{Action, GovernancePacket},
    Chain,
};
use serde_wormhole::RawMessage;
use wormhole_bindings::WormholeQuery;
use std::str;

use crate::{
    msg::{GatewayIbcTokenBridgePayload, COMPLETE_TRANSFER_REPLY_ID},
    state::{CURRENT_TRANSFER, CW_DENOMS, TOKEN_BRIDGE_CONTRACT, VAA_ARCHIVE, CHAIN_TO_CHANNEL_MAP, WORMHOLE_CONTRACT},
    bindings::TokenFactoryMsg,
};

/// Calls into the wormhole token bridge to complete the payload3 transfer.
pub fn complete_transfer_and_convert(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    vaa: Binary,
) -> Result<Response<TokenFactoryMsg>, anyhow::Error> {
    // get the token bridge contract address from storage
    let token_bridge_contract = TOKEN_BRIDGE_CONTRACT
        .load(deps.storage)
        .context("could not load token bridge contract address")?;

    // craft the token bridge execute message
    // this will be added as a submessage to the response
    let token_bridge_execute_msg = to_binary(&TokenBridgeExecuteMsg::CompleteTransferWithPayload {
        data: vaa.clone(),
        relayer: info.sender.to_string(),
    })
    .context("could not serialize token bridge execute msg")?;

    let sub_msg = SubMsg::reply_on_success(
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: token_bridge_contract.clone(),
            msg: token_bridge_execute_msg,
            funds: vec![],
        }),
        COMPLETE_TRANSFER_REPLY_ID,
    );

    // craft the token bridge query message to parse the payload3 vaa
    let token_bridge_query_msg = to_binary(&TokenBridgeQueryMsg::TransferInfo { vaa })
        .context("could not serialize token bridge transfer_info query msg")?;

    let transfer_info: TransferInfoResponse = deps
        .querier
        .query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: token_bridge_contract,
            msg: token_bridge_query_msg,
        }))
        .context("could not parse token bridge payload3 vaa")?;

    // DEFENSE IN-DEPTH CHECK FOR PAYLOAD3 VAAs
    // ensure that the transfer vaa recipient is this contract.
    // we should never process any VAAs that are not directed to this contract.
    let target_address = (&transfer_info.recipient.as_slice()).get_address(0);
    let recipient = deps.api.addr_humanize(&target_address)?;
    ensure!(
        recipient == env.contract.address,
        "vaa recipient must be this contract"
    );

    // save interim state
    CURRENT_TRANSFER
        .save(deps.storage, &transfer_info)
        .context("failed to save current transfer to storage")?;

    // return the response which will callback to the reply handler on success
    Ok(Response::new()
        .add_submessage(sub_msg)
        .add_attribute("action", "complete_transfer_with_payload")
        .add_attribute(
            "transfer_payload",
            Binary::from(transfer_info.payload).to_base64(),
        ))
}

pub fn convert_and_transfer(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    recipient_chain: u16,
    recipient: Binary,
    fee: Uint128,
) -> Result<Response<TokenFactoryMsg>, anyhow::Error> {
    // load the token bridge contract address
    /*let token_bridge_contract = TOKEN_BRIDGE_CONTRACT
        .load(deps.storage)
        .context("could not load token bridge contract address")?;

    ensure!(info.funds.len() == 1, "no bridging coin included");
    let bridging_coin = info.funds[0].clone();
    let cw20_contract_addr = parse_bank_token_factory_contract(deps, env, bridging_coin.clone())?;

    // batch calls together
    let mut response: Response<WormchainMsg> = Response::new();

    // 1. seimsg::burn for the bank tokens
    response = response.add_message(WormchainMsg::BurnTokens {
        amount: bridging_coin.clone(),
    });

    // 2. cw20::increaseAllowance to the contract address for the token bridge to spend the amount of tokens
    let increase_allowance_msg = to_binary(&Cw20WrappedExecuteMsg::IncreaseAllowance {
        spender: token_bridge_contract.clone(),
        amount: bridging_coin.amount,
        expires: None,
    })
    .context("could not serialize cw20 increase_allowance msg")?;
    response = response.add_message(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: cw20_contract_addr.clone(),
        msg: increase_allowance_msg,
        funds: vec![],
    }));

    // 3. token_bridge::initiate_transfer -- the cw20 tokens will be either burned or transferred to the token_bridge
    let initiate_transfer_msg = to_binary(&TokenBridgeExecuteMsg::InitiateTransfer {
        asset: Asset {
            info: AssetInfo::Token {
                contract_addr: cw20_contract_addr,
            },
            amount: bridging_coin.amount,
        },
        recipient_chain,
        recipient,
        fee,
        nonce: 0,
    })
    .context("could not serialize token bridge initiate_transfer msg")?;
    response = response.add_message(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: token_bridge_contract,
        msg: initiate_transfer_msg,
        funds: vec![],
    }));

    Ok(response)*/
    Ok(Response::new())
}


pub fn convert_bank_to_cw20(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
) -> Result<Response<TokenFactoryMsg>, anyhow::Error> {
    // bank tokens sent to the contract will be in info.funds
    /*ensure!(
        info.funds.len() == 1,
        "info.funds should contain only 1 coin"
    );

    let converting_coin = info.funds[0].clone();
    let cw20_contract_addr = parse_bank_token_factory_contract(deps, env, converting_coin.clone())?;

    // batch calls together
    let mut response: Response<WormchainMsg> = Response::new();

    // 1. seimsg::burn for the bank tokens
    response = response.add_message(WormchainMsg::BurnTokens {
        amount: converting_coin.clone(),
    });

    // 2. cw20::transfer to send back to the msg.sender
    let transfer_msg = to_binary(&Cw20ExecuteMsg::Transfer {
        recipient: info.sender.to_string(),
        amount: converting_coin.amount,
    })
    .context("could not serialize cw20::transfer msg")?;
    response = response.add_message(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: cw20_contract_addr,
        msg: transfer_msg,
        funds: vec![],
    }));

    Ok(response)*/
    Ok(Response::new())
}

/*pub fn parse_bank_token_factory_contract(
    deps: DepsMut,
    env: Env,
    coin: Coin,
) -> Result<String, anyhow::Error> {
    // extract the contract address from the denom of the token that was sent to us
    // if the token is not a factory token created by this contract, return error
    let parsed_denom = coin.denom.split("/").collect::<Vec<_>>();
    ensure!(
        parsed_denom.len() == 3
            && parsed_denom[0] == "factory"
            && parsed_denom[1] == env.contract.address.to_string(),
        "coin is not from the token factory"
    );

    // decode subdenom from base64 => encode as cosmos addr to get contract addr
    let cw20_contract_addr = contract_addr_from_base58(deps.as_ref(), parsed_denom[2])?;

    // validate that the contract does indeed match the stored denom we have for it
    let stored_denom = CW_DENOMS
        .load(deps.storage, cw20_contract_addr.clone())
        .context(
            "a corresponding denom for the extracted contract addr is not contained in storage",
        )?;
    ensure!(
        stored_denom == coin.denom,
        "the stored denom for the contract does not match the actual coin denom"
    );

    Ok(cw20_contract_addr)
}*/

pub fn contract_addr_from_base58(deps: Deps, subdenom: &str) -> Result<String, anyhow::Error> {
    let decoded_addr = bs58::decode(subdenom)
        .into_vec()
        .context(format!("failed to decode base58 subdenom {}", subdenom))?;
    let canonical_addr = Binary::from(decoded_addr);
    deps.api
        .addr_humanize(&canonical_addr.into())
        .map(|a| a.to_string())
        .context(format!("failed to humanize cosmos address {}", subdenom))
}

pub fn submit_update_chain_to_channel_map(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    vaa: Binary,
) -> Result<Response<TokenFactoryMsg>, anyhow::Error> {

    // get the token bridge contract address from storage
    let wormhole_contract = WORMHOLE_CONTRACT
        .load(deps.storage)
        .context("could not load token bridge contract address")?;

    //let vaa: ParsedVAA = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
    let wrapped_vaa: Result<ParsedVAA, StdError> = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
    //let vaa = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: wormhole_contract,
        msg: to_binary(&WormholeQueryMsg::VerifyVAA {
            vaa: vaa.clone(),
            block_time: env.block.time.seconds(),
        })?,
    }));

    let vaa = match wrapped_vaa {
        Ok(parsedVAA) => parsedVAA, 
        Err(error) => {
            return Ok(Response::new()
                .add_attribute("error", "parsed vaa")
                .add_attribute("err_output", error.to_string()));
        }
    };

    // validate this is a governance VAA
    ensure!(
        Chain::from(vaa.emitter_chain) == Chain::Solana
            && vaa.emitter_address == wormhole_sdk::GOVERNANCE_EMITTER.0,
        "not a governance VAA"
    );


    // parse the governance packet
    let govpacket = serde_wormhole::from_slice::<GovernancePacket>(&vaa.payload)
        .context("failed to parse governance packet")?;

    // validate the governance VAA is directed to wormchain
    ensure!(
        govpacket.chain == Chain::Wormchain || govpacket.chain == Chain::Any,
        "this governance VAA is for another chain"
    );

    if VAA_ARCHIVE.has(deps.storage, vaa.hash.as_slice()) {
        bail!("governance vaa already executed");
    }
    VAA_ARCHIVE
        .save(deps.storage, vaa.hash.as_slice(), &true)
        .context("failed to save governance VAA to archive")?;

    // match the governance action and execute the corresponding logic
    match govpacket.action {
        Action::UpdateChannelChain {
            channel_id,
            chain_id,
        } => {
            ensure!(chain_id != Chain::Wormchain, "the wormchain-ibc-receiver contract should not maintain channel mappings to wormchain");

            let channel_id_str =
                str::from_utf8(&channel_id).context("failed to parse channel-id as utf-8")?;
            let channel_id_trimmed = channel_id_str.trim_start_matches(char::from(0));

            // update storage with the mapping
            CHAIN_TO_CHANNEL_MAP
                .save(
                    deps.storage,
                    chain_id.into(),
                    &channel_id_trimmed.to_string(),
                )
                .context("failed to save channel chain")?;
            return Ok(Response::new()
                .add_event(Event::new("UpdateChainToChannelMap")
                    .add_attribute("chain_id", chain_id.to_string())
                    .add_attribute("channel_id", channel_id_trimmed)));
        }
    }
}