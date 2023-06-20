#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo,
    Response, StdResult,
};

use cw2::set_contract_version;

use crate::amount::Amount;
use crate::error::ContractError;
use crate::msg::{
    ExecuteMsg, InitMsg, MigrateMsg, QueryMsg,
};
use crate::state::{
    Config, ADMIN, CONFIG, TOKEN_BRIDGE_CONTRACT,
};
use crate::execute::{
    execute_receive, execute_transfer, complete_transfer_and_send,
};
use crate::query::{
    query_port, query_list, query_channel, query_config,
};
use cw_utils::one_coin;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:ibc-gateway";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InitMsg,
) -> Result<Response, anyhow::Error> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let cfg = Config {
        default_timeout: msg.default_timeout,
    };
    CONFIG.save(deps.storage, &cfg)?;

    ADMIN.set(deps.branch(), Some(info.sender))?;

    TOKEN_BRIDGE_CONTRACT
        .save(deps.storage, &msg.token_bridge_contract)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, anyhow::Error> {
    match msg {
        ExecuteMsg::Receive(msg) => execute_receive(deps, env, info, msg),
        ExecuteMsg::Transfer(msg) => {
            let coin = one_coin(&info)?;
            execute_transfer(deps, env, msg, Amount::Native(coin), info.sender)
        }
        ExecuteMsg::UpdateAdmin { admin } => {
            let admin = deps.api.addr_validate(&admin)?;
            Ok(ADMIN.execute_update_admin(deps, info, Some(admin))?)
        }
        ExecuteMsg::CompleteTransferAndSend { vaa } => complete_transfer_and_send(deps, env, info, vaa),
    }
}

/// Reply handler for various kinds of replies
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> Result<Response<SeiMsg>, anyhow::Error> {
    // handle submessage cases based on the reply id
    if msg.id == COMPLETE_TRANSFER_REPLY_ID {
        return handle_complete_transfer_reply(deps, env, msg);
    }

    // other cases probably from calling into the sei burn/mint messages and token factory methods

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Port {} => to_binary(&query_port(deps)?),
        QueryMsg::ListChannels {} => to_binary(&query_list(deps)?),
        QueryMsg::Channel { id } => to_binary(&query_channel(deps, id)?),
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::Admin {} => to_binary(&ADMIN.query_admin(deps)?),
    }
}

/*#[cfg(test)]
mod test {
    use super::*;
    use crate::test_helpers::*;
    use crate::msg::{
        ListChannelsResponse, ChannelResponse, TransferMsg
    };
    use crate::ibc::Ics20Packet;

    use cosmwasm_schema::cw_serde;
    use cosmwasm_std::testing::{mock_env, mock_info};
    use cosmwasm_std::{coin, coins, from_binary, CosmosMsg, IbcMsg, StdError, Uint128, };

    use cw_utils::PaymentError;
    use cw20::Cw20ReceiveMsg;

    #[test]
    fn setup_and_query() {
        let deps = setup(&["channel-3", "channel-7"]);

        let raw_list = query(deps.as_ref(), mock_env(), QueryMsg::ListChannels {}).unwrap();
        let list_res: ListChannelsResponse = from_binary(&raw_list).unwrap();
        assert_eq!(2, list_res.channels.len());
        assert_eq!(mock_channel_info("channel-3"), list_res.channels[0]);
        assert_eq!(mock_channel_info("channel-7"), list_res.channels[1]);

        let raw_channel = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::Channel {
                id: "channel-3".to_string(),
            },
        )
        .unwrap();
        let chan_res: ChannelResponse = from_binary(&raw_channel).unwrap();
        assert_eq!(chan_res.info, mock_channel_info("channel-3"));
        assert_eq!(0, chan_res.total_sent.len());
        assert_eq!(0, chan_res.balances.len());

        let err = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::Channel {
                id: "channel-10".to_string(),
            },
        )
        .unwrap_err();
        assert_eq!(err, StdError::not_found("cw20_ics20::state::ChannelInfo"));
    }

    #[test]
    fn proper_checks_on_execute_native() {
        let send_channel = "channel-5";
        let mut deps = setup(&[send_channel, "channel-10"]);

        let mut transfer = TransferMsg {
            channel: send_channel.to_string(),
            remote_address: "foreign-address".to_string(),
            timeout: None,
            memo: None,
        };

        // works with proper funds
        let msg = ExecuteMsg::Transfer(transfer.clone());
        let info = mock_info("foobar", &coins(1234567, "ucosm"));
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(res.messages[0].gas_limit, None);
        assert_eq!(1, res.messages.len());
        if let CosmosMsg::Ibc(IbcMsg::SendPacket {
            channel_id,
            data,
            timeout,
        }) = &res.messages[0].msg
        {
            let expected_timeout = mock_env().block.time.plus_seconds(DEFAULT_TIMEOUT);
            assert_eq!(timeout, &expected_timeout.into());
            assert_eq!(channel_id.as_str(), send_channel);
            let msg: Ics20Packet = from_binary(data).unwrap();
            assert_eq!(msg.amount, Uint128::new(1234567));
            assert_eq!(msg.denom.as_str(), "ucosm");
            assert_eq!(msg.sender.as_str(), "foobar");
            assert_eq!(msg.receiver.as_str(), "foreign-address");
        } else {
            panic!("Unexpected return message: {:?}", res.messages[0]);
        }

        // reject with no funds
        let msg = ExecuteMsg::Transfer(transfer.clone());
        let info = mock_info("foobar", &[]);
        let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
        assert_eq!(err, ContractError::Payment(PaymentError::NoFunds {}));

        // reject with multiple tokens funds
        let msg = ExecuteMsg::Transfer(transfer.clone());
        let info = mock_info("foobar", &[coin(1234567, "ucosm"), coin(54321, "uatom")]);
        let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
        assert_eq!(err, ContractError::Payment(PaymentError::MultipleDenoms {}));

        // reject with bad channel id
        transfer.channel = "channel-45".to_string();
        let msg = ExecuteMsg::Transfer(transfer);
        let info = mock_info("foobar", &coins(1234567, "ucosm"));
        let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
        assert_eq!(
            err,
            ContractError::NoSuchChannel {
                id: "channel-45".to_string()
            }
        );
    }

    #[test]
    fn proper_checks_on_execute_cw20() {
        let send_channel = "channel-15";
        let cw20_addr = "my-token";
        let mut deps = setup(&["channel-3", send_channel]);

        let transfer = TransferMsg {
            channel: send_channel.to_string(),
            remote_address: "foreign-address".to_string(),
            timeout: Some(7777),
            memo: None,
        };
        let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
            sender: "my-account".into(),
            amount: Uint128::new(888777666),
            msg: to_binary(&transfer).unwrap(),
        });

        // works with proper funds
        let info = mock_info(cw20_addr, &[]);
        let res = execute(deps.as_mut(), mock_env(), info, msg.clone()).unwrap();
        assert_eq!(1, res.messages.len());
        assert_eq!(res.messages[0].gas_limit, None);
        if let CosmosMsg::Ibc(IbcMsg::SendPacket {
            channel_id,
            data,
            timeout,
        }) = &res.messages[0].msg
        {
            let expected_timeout = mock_env().block.time.plus_seconds(7777);
            assert_eq!(timeout, &expected_timeout.into());
            assert_eq!(channel_id.as_str(), send_channel);
            let msg: Ics20Packet = from_binary(data).unwrap();
            assert_eq!(msg.amount, Uint128::new(888777666));
            assert_eq!(msg.denom, format!("cw20:{}", cw20_addr));
            assert_eq!(msg.sender.as_str(), "my-account");
            assert_eq!(msg.receiver.as_str(), "foreign-address");
        } else {
            panic!("Unexpected return message: {:?}", res.messages[0]);
        }

        // reject with tokens funds
        let info = mock_info("foobar", &coins(1234567, "ucosm"));
        let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
        assert_eq!(err, ContractError::Payment(PaymentError::NonPayable {}));
    }

    #[test]
    fn execute_cw20_fails_if_not_whitelisted_unless_default_gas_limit() {
        let send_channel = "channel-15";
        let mut deps = setup(&[send_channel]);

        let cw20_addr = "my-token";
        let transfer = TransferMsg {
            channel: send_channel.to_string(),
            remote_address: "foreign-address".to_string(),
            timeout: Some(7777),
            memo: None,
        };
        let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
            sender: "my-account".into(),
            amount: Uint128::new(888777666),
            msg: to_binary(&transfer).unwrap(),
        });

        // rejected as not on allow list
        let info = mock_info(cw20_addr, &[]);
        let err = execute(deps.as_mut(), mock_env(), info.clone(), msg.clone()).unwrap_err();
        assert_eq!(err, ContractError::NotOnAllowList);

        // add a default gas limit
        migrate(
            deps.as_mut(),
            mock_env(),
            MigrateMsg {
                default_gas_limit: Some(123456),
            },
        )
        .unwrap();

        // try again
        execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    }

    fn test_with_memo(memo: &str) {
        let send_channel = "channel-5";
        let mut deps = setup(&[send_channel, "channel-10"]);

        let transfer = TransferMsg {
            channel: send_channel.to_string(),
            remote_address: "foreign-address".to_string(),
            timeout: None,
            memo: Some(memo.to_string()),
        };

        // works with proper funds
        let msg = ExecuteMsg::Transfer(transfer);
        let info = mock_info("foobar", &coins(1234567, "ucosm"));
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(res.messages[0].gas_limit, None);
        assert_eq!(1, res.messages.len());
        if let CosmosMsg::Ibc(IbcMsg::SendPacket {
            channel_id,
            data,
            timeout,
        }) = &res.messages[0].msg
        {
            let expected_timeout = mock_env().block.time.plus_seconds(DEFAULT_TIMEOUT);
            assert_eq!(timeout, &expected_timeout.into());
            assert_eq!(channel_id.as_str(), send_channel);
            let msg: Ics20Packet = from_binary(data).unwrap();
            assert_eq!(msg.amount, Uint128::new(1234567));
            assert_eq!(msg.denom.as_str(), "ucosm");
            assert_eq!(msg.sender.as_str(), "foobar");
            assert_eq!(msg.receiver.as_str(), "foreign-address");
            assert_eq!(
                msg.memo
                    .expect("Memo was None when Some was expected")
                    .as_str(),
                memo
            );
        } else {
            panic!("Unexpected return message: {:?}", res.messages[0]);
        }
    }

    #[test]
    fn execute_with_memo_works() {
        test_with_memo("memo");
    }

    #[test]
    fn execute_with_empty_string_memo_works() {
        test_with_memo("");
    }

    #[test]
    fn memo_is_backwards_compatible() {
        let mut deps = setup(&["channel-5", "channel-10"]);
        let transfer: TransferMsg = cosmwasm_std::from_slice(
            br#"{"channel": "channel-5", "remote_address": "foreign-address"}"#,
        )
        .unwrap();

        let msg = ExecuteMsg::Transfer(transfer);
        let info = mock_info("foobar", &coins(1234567, "ucosm"));
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(1, res.messages.len());
        if let CosmosMsg::Ibc(IbcMsg::SendPacket {
            channel_id: _,
            data,
            timeout: _,
        }) = &res.messages[0].msg
        {
            let msg: Ics20Packet = from_binary(data).unwrap();
            assert_eq!(msg.memo, None);

            // This is the old version of the Ics20Packet. Deserializing into it
            // should still work as the memo isn't included
            #[cw_serde]
            struct Ics20PacketNoMemo {
                pub amount: Uint128,
                pub denom: String,
                pub sender: String,
                pub receiver: String,
            }

            let _msg: Ics20PacketNoMemo = from_binary(data).unwrap();
        } else {
            panic!("Unexpected return message: {:?}", res.messages[0]);
        }
    }
}*/
