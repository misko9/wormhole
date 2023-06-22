use cosmwasm_std::{CosmosMsg, CustomMsg};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};


// implement custom query
impl CustomMsg for WormchainMsg {}

// this is a helper to be able to return these as CosmosMsg easier
impl From<WormchainMsg> for CosmosMsg<WormchainMsg> {
    fn from(original: WormchainMsg) -> Self {
        CosmosMsg::Custom(original)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum WormchainMsg {
    CreateDenom {
        subdenom: String,
        metadata: Metadata,
    },
    MintTokens {
        denom: String,
        amount: u128,
        mint_to_address: String,
    },
    BurnTokens {
        denom: String,
        amount: u128,
        burn_from_address: String,
    },
    ChangeAdmin {
        denom: String,
        new_admin_address: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Metadata {
    pub description: String,
    pub denom_units: Vec<DenomUnit>,
    pub base: String,
    pub display: String,
    pub name: String,
    pub symbol: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct DenomUnit {
    pub denom: String,
    pub exponent: u32,
    pub aliases: Vec<String>,
}
