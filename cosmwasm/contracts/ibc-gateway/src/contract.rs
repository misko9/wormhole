#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use anyhow::Context;
use cosmwasm_std::{
    DepsMut, Empty, Env,
    MessageInfo, Reply, Response,
};
use cw20::Cw20ReceiveMsg;

use crate::{
    bindings::WormchainMsg,
    msg::{ExecuteMsg, InstantiateMsg, COMPLETE_TRANSFER_REPLY_ID},
    state::TOKEN_BRIDGE_CONTRACT,
    reply::handle_complete_transfer_reply,
    execute::{complete_transfer_and_convert, convert_and_transfer, convert_bank_to_cw20}
};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, anyhow::Error> {
    TOKEN_BRIDGE_CONTRACT
        .save(deps.storage, &msg.token_bridge_contract)
        .context("failed to save token bridge contract address to storage")?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("owner", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: Empty) -> Result<Response, anyhow::Error> {
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response<WormchainMsg>, anyhow::Error> {
    match msg {
        ExecuteMsg::CompleteTransferAndConvert { vaa } => {
            complete_transfer_and_convert(deps, env, info, vaa)
        }
        ExecuteMsg::ConvertAndTransfer {
            recipient_chain,
            recipient,
            fee,
        } => convert_and_transfer(deps, info, env, recipient_chain, recipient, fee),
        ExecuteMsg::ConvertBankToCw20 {} => convert_bank_to_cw20(deps, info, env),
        ExecuteMsg::Receive(Cw20ReceiveMsg {
            sender,
            amount,
            msg,
        }) => Ok(Response::new()),
    }
}

/// Reply handler for various kinds of replies
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> Result<Response<WormchainMsg>, anyhow::Error> {
    // handle submessage cases based on the reply id
    if msg.id == COMPLETE_TRANSFER_REPLY_ID {
        return handle_complete_transfer_reply(deps, env, msg);
    }

    // other cases probably from calling into the sei burn/mint messages and token factory methods
    Ok(Response::default())
}
