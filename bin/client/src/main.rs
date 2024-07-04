//! A simple program to be proven inside the zkVM.

#![no_main]

use guest_primitives::witness::WitnessDb;
sp1_zkvm::entrypoint!(main);

pub fn main() {
    let db = sp1_zkvm::io::read::<WitnessDb>();

    // TODO: Do something useful

    sp1_zkvm::io::commit(&db.state_root);
}
