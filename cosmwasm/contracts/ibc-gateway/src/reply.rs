#[cfg(not(feature = "library"))]
use anyhow::{ensure, Context};
use cosmwasm_std::{
    coin, from_binary, BankMsg, Deps, DepsMut, Env, Reply, Response,
};
use cw_token_bridge::msg::{
    CompleteTransferResponse,
};

use crate::{
    msg::GatewayIbcTokenBridgePayload,
    state::{CURRENT_TRANSFER, CW_DENOMS},
    bindings::{WormchainMsg, Metadata, DenomUnit},
};

pub fn handle_complete_transfer_reply(
    deps: DepsMut,
    env: Env,
    msg: Reply,
) -> Result<Response<WormchainMsg>, anyhow::Error> {
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
        GatewayIbcTokenBridgePayload::Simple { chain, recipient, fee, nonce } => {
            let recipient_decoded = String::from_utf8(recipient.to_vec()).context(format!(
                "failed to convert {} to utf8 string",
                recipient.to_string()
            ))?;
            convert_cw20_to_bank(
                deps,
                env,
                recipient_decoded,
                res_data.amount.into(),
                contract_addr,
            )
        },
        GatewayIbcTokenBridgePayload::ContractControlled { chain, contract, payload, fee, nonce } => {
            let contract_decoded = String::from_utf8(contract.to_vec()).context(format!(
                "failed to convert {} to utf8 string",
                contract.to_string()
            ))?;
            convert_cw20_to_bank(
                deps,
                env,
                contract_decoded,
                res_data.amount.into(),
                contract_addr,
            )
        }
    }
}
pub fn convert_cw20_to_bank(
    deps: DepsMut,
    env: Env,
    recipient: String,
    amount: u128,
    contract_addr: String,
) -> Result<Response<WormchainMsg>, anyhow::Error> {
    // check the recipient and contract addresses are valid
    deps.api
        .addr_validate(&recipient)
        .context(format!("invalid recipient address {}", recipient))?;

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

    let mut response: Response<WormchainMsg> = Response::new();

    // check contract storage see if we've created a denom for this cw20 token yet
    // if we haven't created the denom, then create the denom
    // info.sender contains the cw20 contract address
    if !CW_DENOMS.has(deps.storage, contract_addr.clone()) {
        // call into token factory to create the denom
        response = response.add_message(WormchainMsg::CreateDenom {
            subdenom: tokenfactory_denom.clone(),
            //subdenom: subdenom.clone(),
            metadata: Metadata{
                description: tokenfactory_denom.clone(),
                denom_units: Vec::<DenomUnit>::new(),
                base: subdenom.clone(),
                display: tokenfactory_denom.clone(),
                name: tokenfactory_denom.clone(),
                symbol: subdenom.clone(),
            },
        });

        // add the contract_addr => tokenfactory denom to storage
        CW_DENOMS
            .save(deps.storage, contract_addr, &tokenfactory_denom)
            .context("failed to save contract_addr => tokenfactory denom to storage")?;
    }

    // amount of tokenfactory coins to mint + send
    //let amount = coin(amount, tokenfactory_denom);

    // add calls to mint and send bank tokens
    response = response.add_message(WormchainMsg::MintTokens {
        denom: tokenfactory_denom.clone(),
        amount: amount.clone(),
        mint_to_address: env.contract.address.to_string(),
    });
    /*response = response.add_message(BankMsg::Send {
        to_address: recipient,
        amount: vec![amount],
    });*/

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
