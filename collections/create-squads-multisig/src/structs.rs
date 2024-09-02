use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

#[derive(BorshSerialize, BorshDeserialize)]
pub struct CreateInstructionArgs {
    pub threshold: u16,
    pub create_key: Pubkey,
    pub members: Vec<Pubkey>,
    pub meta: String,
}

#[derive(Serialize, Deserialize)]
pub struct SquadsMetadata {
    pub name: String,
    pub description: String,
    pub image: String,
}