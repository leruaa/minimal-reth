//! Setup inputs to send to the client

use std::fs::File;

use alloy_primitives::B256;
use guest_primitives::witness::WitnessDb;
use sp1_sdk::{ProverClient, SP1Stdin};

/// The ELF we want to execute inside the zkVM.
const ELF: &[u8] = include_bytes!("../../client/elf/riscv32im-succinct-zkvm-elf");

fn main() -> eyre::Result<()> {
    // TODO: read the db JSON file from CLI arg.
    let db = serde_json::from_reader::<_, WitnessDb>(File::open("../")?)?;

    // Setup the logger.
    sp1_sdk::utils::setup_logger();

    // Setup the prover client.
    let client = ProverClient::new();

    // Setup the program.
    let (pk, vk) = client.setup(ELF);

    // Setup the inputs.
    let mut stdin = SP1Stdin::new();
    stdin.write(&db);

    let mut proof = client.prove(&pk, stdin).unwrap();

    println!("generated proof");

    let state = proof.public_values.read::<B256>();

    println!("state: {}", state);

    Ok(())
}
