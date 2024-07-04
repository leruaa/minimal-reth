//! A simple host program that takes in a block number and RPC, and executes the block while
//! fetching state from an RpcDb.
use std::collections::{BTreeMap, HashSet};
use std::fs::File;
use std::io::BufWriter;

use alloy_primitives::Bytes;
use alloy_provider::Provider;
use alloy_provider::{Provider as AlloyProvider, ReqwestProvider};
use alloy_rlp::{Decodable, Encodable};
use eyre::Ok;
use reth_chainspec::ChainSpecBuilder;
use reth_evm::execute::{BlockExecutionOutput, BlockExecutorProvider, Executor};
use reth_evm_ethereum::execute::EthExecutorProvider;
use reth_evm_ethereum::EthEvmConfig;
use reth_execution_errors::BlockValidationError;
use reth_primitives::{Address, Block as RethBlock, Receipts, B256};
use revm::db::CacheDB;
use revm_primitives::{keccak256, Bytecode, HashMap, U256};
use std::sync::Arc;
use url::Url;

use host_primitives::RpcDb;

// TODO: make this a CLI that takes in the block_number and reads in the RPC_URL from a file.
#[tokio::main]
async fn main() -> eyre::Result<()> {
    let block_number = 18884864u64;
    let rpc_url =
        Url::parse("https://eth-mainnet.g.alchemy.com/v2/hIxcf_hqT9It2hS8iCFeHKklL8tNyXNF")
            .expect("Invalid RPC URL");

    println!("Fetching block number {} from {}", block_number, rpc_url);
    // Initialize a provider.
    let provider = ReqwestProvider::new_http(rpc_url);
    let merkle_block_td = U256::ZERO;
    // provider.header_td_by_number(block_number)?.unwrap_or_default();

    let alloy_block = provider
        .get_block_by_number(block_number.into(), true)
        .await?
        .ok_or(eyre::eyre!("block not found"))?;

    let block = RethBlock::try_from(alloy_block)?;
    for transaction in &block.body {
        println!("Transaction: {:?}", transaction);
    }

    let chain_spec = Arc::new(ChainSpecBuilder::mainnet().build());

    let prev_alloy_block = provider
        .get_block_by_number((block_number - 1).into(), true)
        .await?
        .ok_or(eyre::eyre!("prev_block not found"))?;
    let prev_block = RethBlock::try_from(prev_alloy_block)?;
    let prev_state_root = prev_block.header.state_root;

    let cache_provider = provider.clone();
    let provider_db = RpcDb::new(
        cache_provider.clone(),
        (block_number - 1).into(),
        prev_state_root.into(),
    );
    // The reason we can clone the provider_db is all the stateful elements are within Arcs.
    let db = CacheDB::new(provider_db.clone());

    println!("Executing block with provider db...");
    let executor = EthExecutorProvider::new(chain_spec, EthEvmConfig::default()).executor(db);
    let BlockExecutionOutput {
        state, receipts, ..
    } = executor.execute(
        (
            &block
                .clone()
                .with_recovered_senders()
                .ok_or(BlockValidationError::SenderRecoveryError)?,
            (merkle_block_td + block.header.difficulty).into(),
        )
            .into(),
    )?;

    let witness_db = provider_db.into_witness_db();

    let mut witness_db_file = File::create("./db.json")?;
    serde_json::to_writer_pretty(&mut witness_db_file, &witness_db)?;

    // TODO: construct the new block header from this information

    Ok(())
}
