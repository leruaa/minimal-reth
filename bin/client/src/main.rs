//! A simple program to be proven inside the zkVM.

#![no_main]

use std::collections::HashMap;

use revm_primitives::{Address, Bytecode, B256, U256};
use serde::{Deserialize, Serialize};

sp1_zkvm::entrypoint!(main);

#[derive(Debug, Serialize, Deserialize)]
pub struct WitnessDb {
    pub address_to_account_info: HashMap<Address, AccountInfo>,
    pub address_to_storage: HashMap<Address, HashMap<U256, U256>>,
    pub block_hashes: HashMap<U256, B256>,
    pub state_root: B256,
}

/// AccountInfo account information.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AccountInfo {
    /// Account balance.
    pub balance: U256,
    /// Account nonce.
    pub nonce: u64,
    /// code hash,
    pub code_hash: B256,
    /// code: if None, `code_by_hash` will be used to fetch it if code needs to be loaded from
    /// inside of `revm`.
    pub code: Option<Bytecode>,
}

pub fn main() {
    let db = sp1_zkvm::io::read::<WitnessDb>();

    // TODO: Do something useful

    sp1_zkvm::io::commit(&db.state_root);
}
