use cosmwasm_schema::{cw_serde, QueryResponses};
use cw20::Cw20ReceiveMsg;
use cosmwasm_std::Binary;

use crate::amount::Amount;
use crate::state::ChannelInfo;

#[cw_serde]
pub struct InitMsg {
    /// Default timeout for ics20 packets, specified in seconds
    pub default_timeout: u64,
    pub token_bridge_contract: String,
}

#[cw_serde]
pub struct MigrateMsg {
    pub default_gas_limit: Option<u64>,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// This accepts a properly-encoded ReceiveMsg from a cw20 contract
    Receive(Cw20ReceiveMsg),
    /// This allows us to transfer *exactly one* native token
    Transfer(TransferMsg),
    /// Change the admin (must be called by current admin)
    UpdateAdmin { admin: String },
    CompleteTransferAndSend { vaa: Binary },
}

#[cw_serde]
pub enum GatewayIbcTokenBridgePayload {
    Simple { chain: u16, recipient: Binary },
	ContractControlled { chain: u16, contract: Binary, payload: Binary }
}

/// This is the message we accept via Receive
#[cw_serde]
pub struct TransferMsg {
    /// The local channel to send the packets on
    pub channel: String,
    /// The remote address to send to.
    /// Don't use HumanAddress as this will likely have a different Bech32 prefix than we use
    /// and cannot be validated locally
    pub remote_address: String,
    /// How long the packet lives in seconds. If not specified, use default_timeout
    pub timeout: Option<u64>,
    /// An optional memo to add to the IBC transfer
    pub memo: Option<String>,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Return the port ID bound by this contract.
    #[returns(PortResponse)]
    Port {},
    /// Show all channels we have connected to.
    #[returns(ListChannelsResponse)]
    ListChannels {},
    /// Returns the details of the name channel, error if not created.
    #[returns(ChannelResponse)]
    Channel { id: String },
    /// Show the Config.
    #[returns(ConfigResponse)]
    Config {},
    #[returns(cw_controllers::AdminResponse)]
    Admin {},
}

#[cw_serde]
pub struct ListChannelsResponse {
    pub channels: Vec<ChannelInfo>,
}

#[cw_serde]
pub struct ChannelResponse {
    /// Information on the channel's connection
    pub info: ChannelInfo,
    /// How many tokens we currently have pending over this channel
    pub balances: Vec<Amount>,
    /// The total number of tokens that have been sent over this channel
    /// (even if many have been returned, so balance is low)
    pub total_sent: Vec<Amount>,
}

#[cw_serde]
pub struct PortResponse {
    pub port_id: String,
}

#[cw_serde]
pub struct ConfigResponse {
    pub default_timeout: u64,
    pub owner: String,
}
