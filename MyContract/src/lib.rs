use near_contract_standards::fungible_token::metadata::{
    FungibleTokenMetadata, FungibleTokenMetadataProvider, FT_METADATA_SPEC,
};
use near_contract_standards::fungible_token::FungibleToken;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize, to_vec};
use near_sdk::collections::LazyOption;
use near_sdk::json_types::U128;
use near_sdk::{env, log, near_bindgen, AccountId, Balance, PanicOnDefault, PromiseOrValue};
use near_sdk::store::LookupMap;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct MyContract {
    records: LookupMap<String, String>,
}

impl Default for MyContract {
    fn default() -> Self {
        Self {
            records: LookupMap::new(b"r".to_vec())
        }
    }
}

#[near_bindgen]
impl MyContract {
    pub fn set_status(&mut self, message: String) {
        let account_id = env::signer_account_id();
        self.records.insert(account_id.to_string(), message);
    }

    pub fn get_status(&self, account_id: String) -> Option<String> {
        self.get_status(account_id)
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::MockedBlockchain;
    use near_sdk::{testing_env, Balance};

    use super::*;

    fn get_context(predecessor_account_id: AccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            .current_account_id(accounts(0))
            .signer_account_id(predecessor_account_id.clone())
            .predecessor_account_id(predecessor_account_id);
        builder
    }

    #[test]
    fn set_message() {
        let context = get_context(accounts(1));
        testing_env!(context.build()); //初始化一个合约交互的 MockBlockChain 实例
        let mut contract = MyContract::default();
        contract.set_status("lillard".to_string());
        assert_eq!("lillard".to_string(),contract.get_status("bob_near".to_string()))
    }
}