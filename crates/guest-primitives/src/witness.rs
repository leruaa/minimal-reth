use reth_primitives::B256;
use reth_storage_errors::provider::ProviderError;
use revm::DatabaseRef;
use revm_primitives::{AccountInfo, Address, Bytecode, HashMap, U256};

pub struct WitnessDb {
    pub address_to_account_info: HashMap<Address, AccountInfo>,
    pub address_to_storage: HashMap<Address, HashMap<U256, U256>>,
    pub block_hashes: HashMap<U256, B256>,
    pub state_root: B256,
}

impl DatabaseRef for WitnessDb {
    type Error = ProviderError;

    fn basic_ref(&self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        Ok(self.address_to_account_info.get(&address).cloned())
    }

    fn code_by_hash_ref(&self, _code_hash: B256) -> Result<Bytecode, Self::Error> {
        // We return the code from the basic_ref.
        unimplemented!()
    }

    fn storage_ref(&self, address: Address, index: U256) -> Result<U256, Self::Error> {
        Ok(self
            .address_to_storage
            .get(&address)
            .unwrap()
            .get(&index)
            .unwrap()
            .clone())
    }

    fn block_hash_ref(&self, number: U256) -> Result<B256, Self::Error> {
        Ok(self.block_hashes.get(&number).unwrap().clone())
    }
}
