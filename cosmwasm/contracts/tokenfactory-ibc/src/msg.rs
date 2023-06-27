use cosmwasm_schema::cw_serde;


#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    CreateTokenAndSend {
        contract: String,
        amount: u128,
        payload: String,
    }
}