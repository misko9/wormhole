#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use cosmwasm_std::{
    DepsMut, Empty, Env,
    MessageInfo, Response, Reply,
};

use crate::{
    bindings::TokenFactoryMsg,
    msg::{ExecuteMsg, InstantiateMsg},
    execute::create_token_and_send,
};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, anyhow::Error> {
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
) -> Result<Response<TokenFactoryMsg>, anyhow::Error> {
    match msg {
        ExecuteMsg::CreateTokenAndSend { contract, amount, payload } => 
            create_token_and_send(deps, env, info, contract, amount, payload),
    }
}

/// Reply handler for various kinds of replies
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(_deps: DepsMut, _env: Env, _msg: Reply) -> Result<Response<TokenFactoryMsg>, anyhow::Error> {
    Ok(Response::default())
}
