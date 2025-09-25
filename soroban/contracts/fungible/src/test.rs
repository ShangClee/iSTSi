#![cfg(test)]

extern crate std;

use soroban_sdk::{ testutils::Address as _, Address, Env, String };

use crate::contract::{ ISatoshi, ISatoshiClient };

#[test]
fn initial_state() {
    let env = Env::default();

    let contract_addr = env.register(ISatoshi, (Address::generate(&env),Address::generate(&env)));
    let client = ISatoshiClient::new(&env, &contract_addr);

    assert_eq!(client.name(), String::from_str(&env, "iSatoshi"));
}

// Add more tests bellow
