pub mod witness;

// #[derive(Debug, Clone)]
// pub struct FullAccountProof {
//     account_proof: AccountProof,
//     code: Bytecode,
// }

// impl FullAccountProof {
//     fn verify(&self, state_root: B256) -> eyre::Result<()> {
//         self.account_proof.verify(state_root)?;
//         // Assert that the code hash matches the code.
//         // TODO: there is an optimization for EMPTY_CODE_HASH If the self.code is empty.
//         let code_hash = keccak256(self.code.bytes());
//         if self.account_proof.info.unwrap().bytecode_hash.unwrap() != code_hash {
//             return Err(eyre::eyre!("Code hash does not match the code"));
//         }
//         Ok(())
//     }
// }

// use reth_evm::execute::{BlockExecutionOutput, BlockExecutorProvider, Executor};

// impl SP1Input {
//     /// Program that verifies the STF, run inside the zkVM.
//     /// TODO: this should perhaps be a functoin on SP1Input.
//     pub fn verify_stf(&self) -> eyre::Result<()> {
//         let chain_spec = ChainSpecBuilder::default()
//             .chain(MAINNET.chain)
//             .genesis(
//                 serde_json::from_str(include_str!(
//                     "../../../crates/ethereum/node/tests/assets/genesis.json"
//                 ))
//                 .unwrap(),
//             )
//             .shanghai_activated()
//             .build();
//         let block = self.block.clone();
//         let merkle_block_td = U256::from(0); // TODO: this should be an input?

//         println!("Instantiating WitnessDb from SP1Input...");
//         let witness_db_inner = WitnessDb::new(self.clone());
//         let witness_db = CacheDB::new(witness_db_inner);
//         println!("Executing block with witness db...");

//         // TODO: can we import `EthExecutorProvider` from reth-evm instead of reth-node-ethereum?
//         let executor = reth_node_ethereum::EthExecutorProvider::ethereum(chain_spec.clone().into())
//             .executor(witness_db);
//         let BlockExecutionOutput {
//             state, receipts, ..
//         } = executor.execute(
//             (
//                 &block
//                     .clone()
//                     .with_recovered_senders()
//                     .ok_or(BlockValidationError::SenderRecoveryError)?,
//                 (merkle_block_td + block.header.difficulty).into(),
//             )
//                 .into(),
//         )?;
//         println!("Done processing block!");
//         // let block_state = BundleStateWithReceipts::new(
//         //     state,
//         //     Receipts::from_block_receipt(receipts),
//         //     block.header.number,
//         // );

//         // TODO: either return or verify the resulting state root.
//         Ok(())
//     }
// }
