//#[cfg(test)]
//mod test_setup {
use std::marker::PhantomData;

use cosmwasm_std::{
    coin,
    testing::{
        mock_dependencies, mock_env, mock_info, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR,
    },
    to_binary, to_vec, Addr, Api, BankMsg, Binary, CanonicalAddr, Coin, ContractResult, CosmosMsg,
    Empty, Env, OwnedDeps, RecoverPubkeyError, Reply, ReplyOn, StdError, StdResult, SubMsgResponse,
    SystemError, SystemResult, VerificationError, WasmMsg, WasmQuery,
};
use wormhole_bindings::WormholeQuery;

pub const SEI_CONTRACT_ADDR: &str =
    "sei1yw4wv2zqg9xkn67zvq3azye0t8h0x9kgyg3d53jym24gxt49vdyswk5upj";
pub const SEI_USER_ADDR: &str = "sei1vhkm2qv784rulx8ylru0zpvyvw3m3cy9x3xyfv";
pub const SEI_CONTRACT_ADDR_BYTES: [u8; 32] = [
    0x23, 0xaa, 0xe6, 0x28, 0x40, 0x41, 0x4d, 0x69, 0xeb, 0xc2, 0x60, 0x23, 0xd1, 0x13, 0x2f, 0x59,
    0xee, 0xf3, 0x16, 0xc8, 0x22, 0x22, 0xda, 0x46, 0x44, 0xda, 0xaa, 0x83, 0x2e, 0xa5, 0x63, 0x49,
];
pub const SEI_USER_ADDR_BYTES: [u8; 20] = [
    0x65, 0xed, 0xb5, 0x01, 0x9e, 0x3d, 0x47, 0xcf, 0x98, 0xe4, 0xf8, 0xf8, 0xf1, 0x05, 0x84, 0x63,
    0xa3, 0xb8, 0xe0, 0x85,
];

// Custom API mock implementation for testing.
// The custom impl helps us with correct addr_validate, addr_canonicalize, and addr_humanize methods for Sei.
#[derive(Clone)]
pub struct CustomApi {
    contract_addr: String,
    user_addr: String,
    contract_addr_bin: Binary,
    user_addr_bin: Binary,
}

impl CustomApi {
    pub fn new(
        contract_addr: &str,
        user_addr: &str,
        contract_addr_bytes: [u8; 32],
        user_addr_bytes: [u8; 20],
    ) -> Self {
        CustomApi {
            contract_addr: contract_addr.to_string(),
            user_addr: user_addr.to_string(),
            contract_addr_bin: Binary::from(contract_addr_bytes),
            user_addr_bin: Binary::from(user_addr_bytes),
        }
    }
}

impl Api for CustomApi {
    fn addr_validate(&self, input: &str) -> StdResult<Addr> {
        if input == self.contract_addr {
            return Ok(Addr::unchecked(self.contract_addr.clone()));
        }

        if input == self.user_addr {
            return Ok(Addr::unchecked(self.user_addr.clone()));
        }

        return Err(StdError::GenericErr {
            msg: "case not found".to_string(),
        });
    }

    fn addr_canonicalize(&self, input: &str) -> StdResult<CanonicalAddr> {
        if input == self.contract_addr {
            return Ok(CanonicalAddr(self.contract_addr_bin.clone()));
        }

        if input == self.user_addr {
            return Ok(CanonicalAddr(self.user_addr_bin.clone()));
        }

        return Err(StdError::GenericErr {
            msg: "case not found".to_string(),
        });
    }

    fn addr_humanize(&self, canonical: &CanonicalAddr) -> StdResult<Addr> {
        if *canonical == self.contract_addr_bin {
            return Ok(Addr::unchecked(self.contract_addr.clone()));
        }

        if *canonical == self.user_addr_bin {
            return Ok(Addr::unchecked(self.user_addr.clone()));
        }

        return Err(StdError::GenericErr {
            msg: "case not found".to_string(),
        });
    }

    fn secp256k1_verify(
        &self,
        message_hash: &[u8],
        signature: &[u8],
        public_key: &[u8],
    ) -> Result<bool, VerificationError> {
        Ok(cosmwasm_crypto::secp256k1_verify(
            message_hash,
            signature,
            public_key,
        )?)
    }

    fn secp256k1_recover_pubkey(
        &self,
        message_hash: &[u8],
        signature: &[u8],
        recovery_param: u8,
    ) -> Result<Vec<u8>, RecoverPubkeyError> {
        let pubkey =
            cosmwasm_crypto::secp256k1_recover_pubkey(message_hash, signature, recovery_param)?;
        Ok(pubkey.to_vec())
    }

    fn ed25519_verify(
        &self,
        message: &[u8],
        signature: &[u8],
        public_key: &[u8],
    ) -> Result<bool, VerificationError> {
        Ok(cosmwasm_crypto::ed25519_verify(
            message, signature, public_key,
        )?)
    }

    fn ed25519_batch_verify(
        &self,
        messages: &[&[u8]],
        signatures: &[&[u8]],
        public_keys: &[&[u8]],
    ) -> Result<bool, VerificationError> {
        Ok(cosmwasm_crypto::ed25519_batch_verify(
            messages,
            signatures,
            public_keys,
        )?)
    }

    fn debug(&self, message: &str) {
        println!("{}", message);
    }
}

pub fn default_custom_mock_deps() -> OwnedDeps<MockStorage, CustomApi, MockQuerier, Empty> {
    OwnedDeps {
        storage: MockStorage::default(),
        api: CustomApi::new(
            SEI_CONTRACT_ADDR,
            SEI_USER_ADDR,
            SEI_CONTRACT_ADDR_BYTES,
            SEI_USER_ADDR_BYTES,
        ),
        querier: MockQuerier::default(),
        custom_query_type: PhantomData,
    }
}

pub fn execute_custom_mock_deps() -> OwnedDeps<MockStorage, CustomApi, MockQuerier, WormholeQuery> {
    OwnedDeps {
        storage: MockStorage::default(),
        api: CustomApi::new(
            SEI_CONTRACT_ADDR,
            SEI_USER_ADDR,
            SEI_CONTRACT_ADDR_BYTES,
            SEI_USER_ADDR_BYTES,
        ),
        querier: MockQuerier::default(),
        custom_query_type: PhantomData,
    }
}



pub fn mock_env_custom_contract(contract_addr: impl Into<String>) -> Env {
    let mut env = mock_env();
    env.contract.address = Addr::unchecked(contract_addr);
    return env;
}
//}