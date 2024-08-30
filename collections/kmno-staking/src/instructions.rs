use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use solana_sdk::{instruction::AccountMeta, instruction::Instruction, pubkey, pubkey::Pubkey};
use spl_associated_token_account::get_associated_token_address;
use spl_token::ID as TOKEN_PROGRAM_ID;

const KMNO_MINT_ADDRESS: Pubkey = pubkey!("KMNo3nJsBXfcpJTVhZcXLW7RmTwTt4GVFE7suUBo9sS");
const KMNO_STAKING_PROGRAM: Pubkey = pubkey!("FarmsPZpWu9i7Kky8tPN37rs2TpmMrAZrC7S7vJa91Hr");
const KMNO_FARM_STATE: Pubkey = pubkey!("2sFZDpBn4sA42uNbAD6QzQ98rPSmqnPyksYe6SJKVvay");
const KMNO_FARM_VAULT: Pubkey = pubkey!("5xpGE38rm4ZqAgQiuocqkw6cM6Cwrwvx6BVJk6i2oKhv");
const KMNO_SCOPE_PRICES: Pubkey = pubkey!("FarmsPZpWu9i7Kky8tPN37rs2TpmMrAZrC7S7vJa91Hr");

pub fn stake_instruction(account_pubkey: Pubkey, amount: u64) -> Instruction {
    let kmno_ata = get_associated_token_address(&account_pubkey, &KMNO_MINT_ADDRESS);

    let seeds: &[&[u8]] = &[&account_pubkey.to_bytes()];
    let (user_state, _vault_bump) = Pubkey::find_program_address(seeds, &KMNO_STAKING_PROGRAM);
    println!("PROGRAM user_state: {}", user_state.to_string());

    let stake_args = StakeInstructionArgs { amount };
    let stake_serialized_args = bincode::serialize(&stake_args).expect("Error serializing args");

    let mut stake_hasher = Sha256::new();
    stake_hasher.update(b"global:stake");
    let stake_result = stake_hasher.finalize();
    let stake_first_8_bytes = &stake_result[..8];

    let mut stake_data = Vec::new();
    stake_data.extend_from_slice(stake_first_8_bytes);
    stake_data.extend_from_slice(&stake_serialized_args);

    let stake_accounts = vec![
        AccountMeta::new_readonly(account_pubkey, true),
        AccountMeta::new(user_state, false),
        AccountMeta::new(KMNO_FARM_STATE, false),
        AccountMeta::new(KMNO_FARM_VAULT, false),
        AccountMeta::new(kmno_ata, false),
        AccountMeta::new_readonly(KMNO_MINT_ADDRESS, false),
        AccountMeta::new_readonly(KMNO_SCOPE_PRICES, false),
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
    ];

    let stake_instruction =
        Instruction::new_with_bytes(KMNO_STAKING_PROGRAM, &stake_data, stake_accounts);

    return stake_instruction;
}

pub fn unstake_instruction(account_pubkey: Pubkey, amount: u64) -> Instruction {
    let seeds: &[&[u8]] = &[&account_pubkey.to_bytes()];
    let (user_state, _vault_bump) = Pubkey::find_program_address(seeds, &KMNO_STAKING_PROGRAM);
    println!("PROGRAM user_state: {}", user_state.to_string());

    let unstake_args = UnstakeInstructionArgs {
        stake_shares_scaled: amount as u128,
    };
    let unstake_serialized_args =
        bincode::serialize(&unstake_args).expect("Error serializing args");

    let mut unstake_data = Sha256::new();
    unstake_data.update(b"global:unstake");
    let unstake_result = unstake_data.finalize();
    let unstake_first_8_bytes = &unstake_result[..8];

    let mut unstake_data = Vec::new();
    unstake_data.extend_from_slice(unstake_first_8_bytes);
    unstake_data.extend_from_slice(&unstake_serialized_args);

    let stake_accounts = vec![
        AccountMeta::new(account_pubkey, true),
        AccountMeta::new(user_state, false),
        AccountMeta::new(KMNO_FARM_STATE, false),
        AccountMeta::new_readonly(KMNO_SCOPE_PRICES, false),
    ];

    let unstake_instruction =
        Instruction::new_with_bytes(KMNO_STAKING_PROGRAM, &unstake_data, stake_accounts);

    return unstake_instruction;
}

#[derive(Serialize, Deserialize)]
pub struct StakeInstructionArgs {
    pub amount: u64,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnstakeInstructionArgs {
    pub stake_shares_scaled: u128,
}
