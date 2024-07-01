use crate::witness::WitnessDb;

use eyre::Ok;
use reth_primitives::{
    trie::AccountProof, Address, Block as RethBlock, ChainSpecBuilder, Receipts, B256, MAINNET,
};
// use reth_provider::BundleStateWithReceipts;
use revm::db::CacheDB;
use revm_primitives::{keccak256, Bytecode, HashMap, U256};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
/// A struct that holds the input for a zkVM program to execute a block.
pub struct SP1Input {
    /// The previous block.
    pub prev_block: RethBlock,
    /// The block that will be executed inside the zkVM program.
    pub block: RethBlock,
    /// Address to merkle proofs.
    pub address_to_proof: HashMap<Address, FullAccountProof>,
    /// Block number to block hash.
    pub block_hashes: HashMap<U256, B256>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FullAccountProof {
    pub account_proof: AccountProof,
    pub code: Bytecode,
}

impl FullAccountProof {
    fn verify(&self, state_root: B256) -> eyre::Result<()> {
        self.account_proof.verify(state_root)?;
        // Assert that the code hash matches the code.
        // TODO: there is an optimization for EMPTY_CODE_HASH If the self.code is empty.
        let code_hash = keccak256(self.code.bytes());
        if self.account_proof.info.unwrap().bytecode_hash.unwrap() != code_hash {
            return Err(eyre::eyre!("Code hash does not match the code"));
        }
        Ok(())
    }
}
