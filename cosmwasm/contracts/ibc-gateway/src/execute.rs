#[cfg(not(feature = "library"))]
use cosmwasm_std::{
    from_binary, to_binary, Addr, Binary, CosmosMsg, DepsMut, Env, IbcMsg, MessageInfo, Response,
    QueryRequest, Reply, SubMsg, WasmMsg, WasmQuery,
};
use cw20::{Cw20Coin, Cw20ReceiveMsg};
use cw_token_bridge::msg::{
    Asset, AssetInfo, CompleteTransferResponse, ExecuteMsg as TokenBridgeExecuteMsg,
    QueryMsg as TokenBridgeQueryMsg, TransferInfoResponse,
};
use cw_wormhole::byte_utils::ByteUtils;
use crate::amount::Amount;
use crate::error::ContractError;
use crate::ibc::Ics20Packet;
use crate::msg::{
    TransferMsg, GatewayIbcTokenBridgePayload,
};
use crate::state::{
    increase_channel_balance, CHANNEL_INFO, CONFIG, TOKEN_BRIDGE_CONTRACT, CURRENT_TRANSFER,
};
use cw_utils::nonpayable;
use anyhow::{ensure, Context};

const COMPLETE_TRANSFER_REPLY_ID: u64 = 1;

pub fn complete_transfer_and_send(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    vaa: Binary,
) -> Result<Response, anyhow::Error> {
    // get the token bridge contract address from storage
    let token_bridge_contract = TOKEN_BRIDGE_CONTRACT
    .load(deps.storage)?;

    // craft the token bridge execute message
    // this will be added as a submessage to the response
    let token_bridge_execute_msg = to_binary(&TokenBridgeExecuteMsg::CompleteTransferWithPayload {
        data: vaa.clone(),
        relayer: info.sender.to_string(),
    })?;

    let sub_msg = SubMsg::reply_on_success(
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: token_bridge_contract.clone(),
            msg: token_bridge_execute_msg,
            funds: vec![],
        }),
        COMPLETE_TRANSFER_REPLY_ID,
    );

    // craft the token bridge query message to parse the payload3 vaa
    let token_bridge_query_msg = to_binary(&TokenBridgeQueryMsg::TransferInfo { vaa })?;

    let transfer_info: TransferInfoResponse = deps
        .querier
        .query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: token_bridge_contract,
            msg: token_bridge_query_msg,
        }))?;

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
        .save(deps.storage, &transfer_info)?;

    // return the response which will callback to the reply handler on success
    Ok(Response::new()
        .add_submessage(sub_msg)
        .add_attribute("action", "complete_transfer_with_payload")
        .add_attribute(
            "transfer_payload",
            Binary::from(transfer_info.payload).to_base64(),
        ))
    //Ok(Response::new())
}

pub fn execute_receive(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    wrapper: Cw20ReceiveMsg,
) -> Result<Response, anyhow::Error> {
    nonpayable(&info)?;

    let msg: TransferMsg = from_binary(&wrapper.msg)?;
    let amount = Amount::Cw20(Cw20Coin {
        address: info.sender.to_string(),
        amount: wrapper.amount,
    });
    let api = deps.api;
    execute_transfer(deps, env, msg, amount, api.addr_validate(&wrapper.sender)?)
}

pub fn execute_transfer(
    deps: DepsMut,
    env: Env,
    msg: TransferMsg,
    amount: Amount,
    sender: Addr,
) -> Result<Response, anyhow::Error> {
    if amount.is_empty() {
        return Err(ContractError::NoFunds {})?;
    }
    // ensure the requested channel is registered
    if !CHANNEL_INFO.has(deps.storage, &msg.channel) {
        return Err(ContractError::NoSuchChannel { id: msg.channel })?;
    }
    let config = CONFIG.load(deps.storage)?;

    // if cw20 token, validate and ensure it is whitelisted, or we set default gas limit
    /*if let Amount::Cw20(coin) = &amount {
        let addr = deps.api.addr_validate(&coin.address)?;
        // if limit is set, then we always allow cw20
        if config.default_gas_limit.is_none() {
            ALLOW_LIST
                .may_load(deps.storage, &addr)?
                .ok_or(ContractError::NotOnAllowList)?;
        }
    };*/

    // delta from user is in seconds
    let timeout_delta = match msg.timeout {
        Some(t) => t,
        None => config.default_timeout,
    };
    // timeout is in nanoseconds
    let timeout = env.block.time.plus_seconds(timeout_delta);

    // build ics20 packet
    let packet = Ics20Packet::new(
        amount.amount(),
        amount.denom(),
        sender.as_ref(),
        &msg.remote_address,
    )
    .with_memo(msg.memo);
    packet.validate()?;

    // Update the balance now (optimistically) like ibctransfer modules.
    // In on_packet_failure (ack with error message or a timeout), we reduce the balance appropriately.
    // This means the channel works fine if success acks are not relayed.
    increase_channel_balance(deps.storage, &msg.channel, &amount.denom(), amount.amount())?;

    // prepare ibc message
    let msg = IbcMsg::SendPacket {
        channel_id: msg.channel,
        data: to_binary(&packet)?,
        timeout: timeout.into(),
    };

    // send response
    let res = Response::new()
        .add_message(msg)
        .add_attribute("action", "transfer")
        .add_attribute("sender", &packet.sender)
        .add_attribute("receiver", &packet.receiver)
        .add_attribute("denom", &packet.denom)
        .add_attribute("amount", &packet.amount.to_string());
    Ok(res)
}

fn handle_complete_transfer_reply(
    deps: DepsMut,
    env: Env,
    msg: Reply,
) -> Result<Response, anyhow::Error> {
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
    let transfer_msg = TransferMsg{
        channel: "channel-0".to_string(),
        remote_address: "".to_string(),
        timeout: None,
        memo: None,
    };
    let amount = Amount::Cw20::C{
        address: "".to_string(),
        amount: transfer_info.amount,
    };
    match payload {
        GatewayIbcTokenBridgePayload::Simple { chain, recipient } => {
            let recipient_decoded = String::from_utf8(recipient.to_vec()).context(format!(
                "failed to convert {} to utf8 string",
                recipient.to_string()
            ))?;
            transfer_msg.remote_address = recipient_decoded;
            execute_transfer(deps, env, transfer_msg, transfer_info.amount, sender)
        },
        GatewayIbcTokenBridgePayload::ContractControlled { chain, contract, payload } => {
                execute_transfer(deps, env, msg, amount, sender)
        },
    }
}
