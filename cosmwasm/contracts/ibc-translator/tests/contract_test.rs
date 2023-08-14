use ibc_translator::{
    execute::{
        complete_transfer_and_convert, convert_and_transfer, TransferType,
    },
    state::{TOKEN_BRIDGE_CONTRACT, CURRENT_TRANSFER, CW_DENOMS},
    msg::COMPLETE_TRANSFER_REPLY_ID,
    contract::reply,
};
use cosmwasm_std::{
    coin, to_binary, Binary, ContractResult, CosmosMsg, Reply, ReplyOn, Response, SubMsgResponse, SystemError, SystemResult, Uint128, WasmMsg, WasmQuery,
    testing::{
        mock_dependencies, mock_env, mock_info,
    },
};

use cw_token_bridge::msg::{AssetInfo, CompleteTransferResponse, TransferInfoResponse};
use wormhole_bindings::tokenfactory::{TokenFactoryMsg, TokenMsg};

mod test_setup;
use test_setup::*;

// TESTS
// 1. instantiate
//    1. happy path
//    2. no storage
// 2. migrate
//    1. happy path
// 3. execute
//    1. CompleteTransferAndConvert
//    2. GatewayConvertAndTransfer
//    3. GatewayConvertAndTransferWithPaylod
//    4. SubmitUpdateChainToChannelMap
// 4. reply
//    1. happy path (done)
//    2. no id match (done)
// 5. query
//    1. happy path

// TESTS: reply
// 1. Happy path: REPLY ID matches
#[test]
fn reply_happy_path() {
    let mut deps = mock_dependencies();
    let env = mock_env();

    // for this test we don't build a proper reply.
    // we're just testing that the handle_complete_transfer_reply method is called when the reply_id is 1
    let msg = Reply {
        id: 1,
        result: cosmwasm_std::SubMsgResult::Err("random error".to_string()),
    };

    let err = reply(deps.as_mut(), env, msg).unwrap_err();
    assert_eq!(
        err.to_string(),
        "msg result is not okay, we should never get here"
    );
}

// 2. ID does not match reply -- no op
#[test]
fn reply_no_id_match() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let msg = Reply {
        id: 0,
        result: cosmwasm_std::SubMsgResult::Ok(SubMsgResponse {
            events: vec![],
            data: None,
        }),
    };

    let err = reply(deps.as_mut(), env, msg).unwrap_err();
    assert_eq!(err.to_string(), "unmatched reply id 0");
}