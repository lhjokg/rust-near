mod errors;
mod action;

use near_contract_standards::fungible_token::metadata::{
    FungibleTokenMetadata, FungibleTokenMetadataProvider, FT_METADATA_SPEC,
};
use near_contract_standards::fungible_token::FungibleToken;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LazyOption;
use near_sdk::json_types::U128;
use near_sdk::{env, log, near_bindgen, AccountId, Balance, PanicOnDefault, PromiseOrValue, Promise, Gas, PromiseError};
use near_sdk::utils::assert_one_yocto;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::serde_json::json;
use near_sdk::{ONE_NEAR, ONE_YOCTO};
use action::{Action};
use std::fmt;
use errors::*;

const XCC_GAS: Gas = Gas(10u64.pow(13));
const GAS: Gas = Gas(10u64.pow(14));

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    /// Account of the owner.
    owner_id: AccountId,
    state: RunningState,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new() -> Self {
        // assert!(!env::state_exists(), "Already initialized");
        Self {
            owner_id: env::current_account_id(),
            state: RunningState::Running,
        }
    }

    #[payable]
    pub fn set_owner(&mut self, owner_id: AccountId) {
        assert_one_yocto();
        self.assert_owner();
        self.owner_id = owner_id.clone();
    }

    //修改数据结构用这个升级
    #[init(ignore_state)]
    pub fn migrate() -> Self {
        let old_state: Contract = env::state_read().expect("failed");
        old_state.assert_owner();
        Self {
            owner_id: old_state.owner_id,
            state: RunningState::Running,
        }
    }

    #[payable]
    pub fn smart_swap(&mut self, from_token: AccountId, to_token: AccountId, amount_in: U128, min_return: U128, exchange: Vec<String>, data: Vec<String>) -> Promise {
        // assert_ne!(actions.len(), 0, "{}", ERR72_AT_LEAST_ONE_SWAP);
        //
        // let user_account = env::predecessor_account_id().to_string();
        //
        // Promise::new(to_token.clone()).function_call(
        //     "ft_balance_of".to_owned(),
        //     json!({ "account_id": user_account }).to_string().into_bytes(),
        //     0,
        //     XCC_GAS,
        // ).then(Self::ext(env::current_account_id()).balance_callback());
        //
        // let balance_before = match env::promise_result(0) {
        //     PromiseResult::NotReady => unreachable!(),
        //     PromiseResult::Successful(value) => {
        //         if let Ok(amount) = near_sdk::serde_json::from_slice::<U128>(&value) {
        //             amount.0
        //         } else {
        //             assert!(false, "{}", ERR41_WRONG_ACTION_RESULT);
        //         }
        //     }
        //     PromiseResult::Failed => assert!(false, "{}", ERR41_WRONG_ACTION_RESULT),
        // };
        // log!("balance_before {}", balance_before);
        // let mut dex_promise = Promise::new(from_token.clone()).function_call(
        //     "ft_transfer_call".to_owned(),
        //     data.get(i).to_string().into_bytes(),
        //     to_yocto("1"),
        //     XCC_GAS,
        // );
        // for i in 1..exchange.len() {
        //     dex_promise.and(Promise::new(from_token.clone()).function_call(
        //         "ft_transfer_call".to_owned(),
        //         data.get(i).to_string().into_bytes(),
        //         to_yocto("1"),
        //         XCC_GAS,
        //     ));
        // }
        //
        //
        // log!("Swapped {} {} to {} {}", amount_in.0, from_token, amount_out.0, to_token);
        // amount_out
        let user_account = env::predecessor_account_id().to_string();
        log!("user_account {}",user_account);
        Promise::new(from_token.clone()).function_call(
            "ft_transfer_call".to_owned(),
            data.get(0).unwrap().to_string().into_bytes(),
            to_yocto("1"),
            XCC_GAS,
        ).function_call(
            "ft_balance_of".to_owned(),
            json!({ "account_id": user_account }).to_string().into_bytes(),
            to_yocto("1"),
            XCC_GAS,
        ).then(Self::ext(env::current_account_id()).balance_callback())
    }

    #[payable]
    pub fn smart_swap1(&mut self, from_token: AccountId, to_token: AccountId, amount_in: U128, min_return: U128, exchange: Vec<String>, data: Vec<String>) -> Promise {
        // amount_out
        let user_account = env::predecessor_account_id().to_string();
        let current_account_id = env::current_account_id();
        log!("user_account {} ,current_account_id {} ",user_account,current_account_id);
        let transfer_promise = Promise::new(from_token.clone())
            .function_call(
                "internal_transfer".to_owned(),
                json!({ "sender_id": user_account, "receiver_id": current_account_id,"amount":amount_in}).to_string().into_bytes(),
                ONE_YOCTO,
                GAS);
        let balancer_before = Promise::new(to_token.clone())
            .function_call(
                "ft_balance_of".to_owned(),
                json!({ "account_id": current_account_id }).to_string().into_bytes(),
                0,
                XCC_GAS);

        let swap_balancer = Promise::new(from_token.clone())
            .function_call(
                "ft_transfer_call".to_owned(),
                data.get(0).unwrap().to_string().into_bytes(),
                ONE_YOCTO,
                GAS);

        let balancer_after = Promise::new(to_token.clone())
            .function_call(
                "ft_balance_of".to_owned(),
                json!({ "account_id": current_account_id }).to_string().into_bytes(),
                0,
                XCC_GAS);

        transfer_promise
            .and(balancer_before)
            .and(swap_balancer)
            .and(balancer_after)
            .then(Self::ext(env::current_account_id()).balance_callback())
    }

    #[private]
    pub fn balance_callback(
        &self,
        #[callback_result] callback_result: Result<U128, PromiseError>,
    ) -> U128 {
        log!("The call results count {}",env::promise_results_count());
        // The callback only has access to the last action's result
        if let Ok(result) = callback_result {
            log!("The call result is {}",result.0);
            result
        } else {
            log!("The call failed and all calls got reverted");
            U128(0)
        }
    }

    pub fn get_owner(&self) -> String {
        self.owner_id.clone().to_string()
    }

    fn assert_owner(&self) {
        assert_eq!(
            env::predecessor_account_id(),
            self.owner_id,
            "{}", "E100: no permission to invoke this"
        );
    }

    fn assert_contract_running(&self) {
        match self.state {
            RunningState::Running => (),
            _ => env::panic_str(ERR51_CONTRACT_PAUSED),
        };
    }
}


pub fn to_yocto(value: &str) -> u128 {
    let vals: Vec<_> = value.split('.').collect();
    let part1 = vals[0].parse::<u128>().unwrap() * 10u128.pow(24);
    if vals.len() > 1 {
        let power = vals[1].len() as u32;
        let part2 = vals[1].parse::<u128>().unwrap() * 10u128.pow(24 - power);
        part1 + part2
    } else {
        part1
    }
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Eq, PartialEq, Clone)]
#[serde(crate = "near_sdk::serde")]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
pub enum RunningState {
    Running,
    Paused,
}

impl fmt::Display for RunningState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RunningState::Running => write!(f, "Running"),
            RunningState::Paused => write!(f, "Paused"),
        }
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::MockedBlockchain;
    use near_sdk::{testing_env, Balance};

    use super::*;

    const TOTAL_SUPPLY: Balance = 1_000_000_000_000_000;

    fn get_context(predecessor_account_id: AccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            .current_account_id(accounts(0))
            .signer_account_id(predecessor_account_id.clone())
            .predecessor_account_id(predecessor_account_id);
        builder
    }

    #[test]
    fn test_new() {
        let mut context = get_context(accounts(1));
        testing_env!(context.build());
        let contract = Contract::new_default_meta(accounts(1).into(), TOTAL_SUPPLY.into());
        testing_env!(context.is_view(true).build());
        assert_eq!(contract.ft_total_supply().0, TOTAL_SUPPLY);
        assert_eq!(contract.ft_balance_of(accounts(1)).0, TOTAL_SUPPLY);
    }

    #[test]
    #[should_panic(expected = "The contract is not initialized")]
    fn test_default() {
        let context = get_context(accounts(1));
        testing_env!(context.build());
        let _contract = Contract::default();
    }

    #[test]
    fn test_transfer() {
        let mut context = get_context(accounts(2));
        testing_env!(context.build());
        let mut contract = Contract::new_default_meta(accounts(2).into(), TOTAL_SUPPLY.into());
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(contract.storage_balance_bounds().min.into())
            .predecessor_account_id(accounts(1))
            .build());
        // Paying for account registration, aka storage deposit
        contract.storage_deposit(None, None);

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(1)
            .predecessor_account_id(accounts(2))
            .build());
        let transfer_amount = TOTAL_SUPPLY / 3;
        contract.ft_transfer(accounts(1), transfer_amount.into(), None);

        testing_env!(context
            .storage_usage(env::storage_usage())
            .account_balance(env::account_balance())
            .is_view(true)
            .attached_deposit(0)
            .build());
        assert_eq!(contract.ft_balance_of(accounts(2)).0, (TOTAL_SUPPLY - transfer_amount));
        assert_eq!(contract.ft_balance_of(accounts(1)).0, transfer_amount);
    }
}
