use futures::future::join_all;
use std::sync::Arc;
use tokio::sync::RwLock;

use alloy_provider::{Provider, ReqwestProvider};
use alloy_rpc_types::{BlockId, EIP1186AccountProofResponse};
use reth_primitives::{Account, Address, B256, U256};
use reth_revm::DatabaseRef;
use reth_storage_errors::provider::ProviderError;
use reth_trie_common::{AccountProof, StorageProof};
use revm_primitives::{hash_map::Entry, AccountInfo, Bytecode, HashMap, HashSet};

// TODO: this should be upstreamed to Reth in the "rpc-types-compat" crate.
pub fn convert_proof(proof: EIP1186AccountProofResponse) -> AccountProof {
    let address = proof.address;
    let balance = proof.balance;
    let code_hash = proof.code_hash;
    let nonce = proof.nonce.as_limbs()[0];
    let storage_hash = proof.storage_hash;
    let account_proof = proof.account_proof;
    let account_info = Account {
        nonce,
        balance,
        bytecode_hash: code_hash.into(),
    };
    let storage_proofs = proof.storage_proof.into_iter().map(|storage_proof| {
        let key = storage_proof.key;
        let value = storage_proof.value;
        let proof = storage_proof.proof;
        let mut sp = StorageProof::new(key.0.into());
        sp.set_value(value);
        sp.set_proof(proof);
        sp
    });
    AccountProof {
        address,
        info: Some(account_info),
        proof: account_proof,
        storage_root: storage_hash.into(),
        storage_proofs: storage_proofs.collect(),
    }
}

#[derive(Clone, Debug)]
/// An implementation of a [`DatabaseRef`] that uses an [`ReqwestProvider`] to fetch data.
pub struct RpcDb {
    /// The [`ReqwestProvider`] that will be used to fetch data.
    /// TODO: In the future this should be a generic [`Provider`] trait from alloy_provider.
    pub provider: ReqwestProvider,
    /// The [`BlockId`] that will be used when fetching data from the RPC.
    pub block: BlockId,
    /// The [`State`]
    pub state_root: B256,
    /// A mapping from [`Address`] to [`AccountInfo`] for all addresses that have been fetched.
    pub address_to_account_info: Arc<RwLock<HashMap<Address, AccountInfo>>>,
    pub address_to_storage: Arc<RwLock<HashMap<Address, HashMap<U256, U256>>>>,
    pub block_hashes: Arc<RwLock<HashMap<U256, B256>>>,
}

impl RpcDb {
    pub fn new(provider: ReqwestProvider, block: BlockId, state_root: B256) -> Self {
        RpcDb {
            provider,
            block,
            state_root,
            address_to_account_info: Arc::new(RwLock::new(HashMap::new())),
            address_to_storage: Arc::new(RwLock::new(HashMap::new())),
            block_hashes: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Fetch the account info (and code) for an address.
    pub async fn fetch_account_info(&self, address: Address) -> AccountInfo {
        let mut address_to_account_info = self.address_to_account_info.write().await;

        match address_to_account_info.entry(address) {
            Entry::Occupied(e) => {
                println!("Retrieving account info for address {address:?} from cache");
                e.get().clone()
            }
            Entry::Vacant(e) => {
                println!("Fetching account info for address {address:?}...");
                // TODO: when alloy adds a `eth_getAccount` method, we can use that here to save RPC load,
                // since getProof is expensive.
                let proof = self
                    .provider
                    .get_proof(address, vec![])
                    .block_id(self.block)
                    .await
                    .expect("Failed to get proof");
                println!("Fetched proof for address {address:?}...");
                let code = self
                    .provider
                    .get_code_at(address)
                    .block_id(self.block)
                    .await
                    .expect("Failed to get code");
                println!("Fetched code for address {address:?}...");
                let bytecode = Bytecode::new_raw(code);

                let account_info = AccountInfo {
                    nonce: proof.nonce.as_limbs()[0],
                    balance: proof.balance,
                    code_hash: proof.code_hash,
                    code: Some(bytecode.clone()),
                };
                println!("Inserting into address_to_account_info...");
                println!("Fetched");
                // Keep track of the account_info and code in the mappings for RpcDb.
                e.insert(account_info).clone()
            }
        }
    }

    /// Fetch the storage for an address and index.
    async fn fetch_storage(&self, address: Address, index: U256) -> U256 {
        let mut address_to_storage = self.address_to_storage.write().await;
        let storage = address_to_storage.entry(address).or_default();

        match storage.entry(index) {
            Entry::Occupied(e) => {
                println!(
                    "Retrieving storage for address {address:?} and index {index:?} from cache"
                );
                *e.get()
            }
            Entry::Vacant(e) => {
                println!("Fetching storage for address {address:?} and index {index:?}...");

                let value = self
                    .provider
                    .get_storage_at(address, index)
                    .block_id(self.block)
                    .await
                    .expect("Failed to get storage");

                *e.insert(value)
            }
        }
    }

    /// Fetch the block hash for a block number.
    async fn fetch_block_hash(&self, number: U256) -> B256 {
        let mut block_hashes = self.block_hashes.write().await;

        match block_hashes.entry(number) {
            Entry::Occupied(e) => {
                println!("Retrieving block hash for number: {:?}from cache", number);
                *e.get()
            }
            Entry::Vacant(e) => {
                println!("Fetching block hash for number: {:?}", number);
                let num_u64 = number.as_limbs()[0];
                let block = self
                    .provider
                    .get_block_by_number(num_u64.into(), false)
                    .await
                    .expect("Failed to get block");
                let hash = block
                    .expect("Block not found")
                    .header
                    .hash
                    .expect("Block hash not found");

                *e.insert(hash)
            }
        }
    }

    /// Given all of the account and storage accesses in a block, fetch merkle proofs for all of
    /// them.
    async fn get_proofs(&self) -> HashMap<Address, EIP1186AccountProofResponse> {
        println!("Fetching proofs...");
        // Acquire read locks at the top
        let account_info = self.address_to_account_info.read().await;
        let storage_guard = self.address_to_storage.read().await;

        let mut addresses: HashSet<&Address> = account_info.keys().collect();
        addresses.extend(storage_guard.keys());

        // Create a future for each address to fetch a proof of the account and storage keys.
        let futures: Vec<_> = addresses
            .into_iter()
            .map(|address| {
                let storage_keys: Vec<B256> = storage_guard
                    .get(address)
                    .map(|storage_map| {
                        storage_map
                            .keys()
                            .into_iter()
                            .map(|k| (*k).into())
                            .collect()
                    })
                    .unwrap_or_else(Vec::new);

                let provider = self.provider.clone();
                async move {
                    match provider
                        .get_proof(*address, storage_keys)
                        .block_id(self.block)
                        .await
                    {
                        Ok(proof) => Some((*address, proof)),
                        Err(_) => None,
                    }
                }
            })
            .collect();

        // Execute all futures in parallel.
        let results = join_all(futures).await;

        // Collect results into a HashMap.
        results.into_iter().filter_map(|result| result).collect()
    }
}

impl DatabaseRef for RpcDb {
    type Error = ProviderError;

    fn basic_ref(&self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            Ok(Some(tokio::task::block_in_place(|| {
                handle.block_on(self.fetch_account_info(address))
            })))
        } else {
            panic!("No tokio runtime found");
        }
    }

    fn code_by_hash_ref(&self, _code_hash: B256) -> Result<Bytecode, Self::Error> {
        unimplemented!()
    }

    fn storage_ref(&self, address: Address, index: U256) -> Result<U256, Self::Error> {
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            Ok(tokio::task::block_in_place(|| {
                handle.block_on(self.fetch_storage(address, index))
            }))
        } else {
            panic!("No tokio runtime found");
        }
    }

    fn block_hash_ref(&self, number: U256) -> Result<B256, Self::Error> {
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            Ok(tokio::task::block_in_place(|| {
                handle.block_on(self.fetch_block_hash(number))
            }))
        } else {
            panic!("No tokio runtime found");
        }
    }
}
