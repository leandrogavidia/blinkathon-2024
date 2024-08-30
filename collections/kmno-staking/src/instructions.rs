use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{instruction::AccountMeta, instruction::Instruction, pubkey, pubkey::Pubkey, system_program::ID as SYSTEM_PROGRAM_ID};
use spl_associated_token_account::get_associated_token_address;
use spl_token::ID as TOKEN_PROGRAM_ID;

const KMNO_MINT_ADDRESS: Pubkey = pubkey!("KMNo3nJsBXfcpJTVhZcXLW7RmTwTt4GVFE7suUBo9sS");
const KMNO_STAKING_PROGRAM: Pubkey = pubkey!("FarmsPZpWu9i7Kky8tPN37rs2TpmMrAZrC7S7vJa91Hr");
const KMNO_FARM_STATE: Pubkey = pubkey!("2sFZDpBn4sA42uNbAD6QzQ98rPSmqnPyksYe6SJKVvay");
const KMNO_FARM_VAULT: Pubkey = pubkey!("5xpGE38rm4ZqAgQiuocqkw6cM6Cwrwvx6BVJk6i2oKhv");
const KMNO_FARM_VAULT_AUTHORITY: Pubkey = pubkey!("Ec6MuWtpvFcVyMsp7vipKCg1CMkKrWHZpWPdnJF16G57");
const KMNO_SCOPE_PRICES: Pubkey = pubkey!("FarmsPZpWu9i7Kky8tPN37rs2TpmMrAZrC7S7vJa91Hr");
const RENT_PROGRAM: Pubkey = pubkey!("SysvarRent111111111111111111111111111111111");
const BASE_SEED_USER_STATE: &[u8; 4] = b"user";
const KMNO_DECIMALS: u32 = 6; 
const KMNO_UNSTAKE_DECIMALS: u32 = 6 * 4;

pub fn initializer_user_instruction(account_pubkey: Pubkey, user_state: Pubkey) -> Instruction {
    let mut stake_hasher = Sha256::new();
    stake_hasher.update(b"global:initialize_user");
    let stake_result = stake_hasher.finalize();
    let stake_first_8_bytes = &stake_result[..8];

    let mut stake_data = Vec::new();
    stake_data.extend_from_slice(stake_first_8_bytes);

    let stake_accounts = vec![
        AccountMeta::new(account_pubkey, true),
        AccountMeta::new(account_pubkey, true),
        AccountMeta::new_readonly(account_pubkey, false),
        AccountMeta::new_readonly(account_pubkey, false),
        AccountMeta::new(user_state, false),
        AccountMeta::new(KMNO_FARM_STATE, false),
        AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
        AccountMeta::new_readonly(RENT_PROGRAM, false),
    ];

    let initializer_user_instruction =
        Instruction::new_with_bytes(KMNO_STAKING_PROGRAM, &stake_data, stake_accounts);

    return initializer_user_instruction;
}

pub async fn stake_instruction(account_pubkey: Pubkey, user_amount: f32, rpc: String) -> Vec<Instruction> {
    let kmno_ata = get_associated_token_address(&account_pubkey, &KMNO_MINT_ADDRESS);

    let (user_state, _user_state_bump) = Pubkey::find_program_address(
        &[
            BASE_SEED_USER_STATE,
            KMNO_FARM_STATE.as_ref(),
            account_pubkey.as_ref(),
        ],
        &KMNO_STAKING_PROGRAM,
    );

    let decimals_result = 10u64.pow(KMNO_DECIMALS);
    let amount: u64 = user_amount as u64 * decimals_result;

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

    let client = RpcClient::new(rpc);
    let account_info = client.get_account(&user_state).await;

    let mut instructions = vec![];

    match account_info {
        Ok(_account) => {
            println!("Account already exist!");
        },
        Err(_error) => {
            let init_user_instruction = initializer_user_instruction(account_pubkey, user_state);
            instructions.push(init_user_instruction);
            println!("Account does not exist!");
        }
    }

    let stake_instruction =
        Instruction::new_with_bytes(KMNO_STAKING_PROGRAM, &stake_data, stake_accounts);

    instructions.push(stake_instruction);

    return instructions;
}

pub fn unstake_instruction(account_pubkey: Pubkey, user_state: Pubkey, user_amount: f32) -> Instruction {
    let decimals_result = 10u128.pow(KMNO_UNSTAKE_DECIMALS);
    let amount: u128 = user_amount as u128 * decimals_result;

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

pub fn withdraw_unstaked_deposits_instruction(account_pubkey: Pubkey, user_amount: f32) -> Vec<Instruction> {
    let kmno_ata = get_associated_token_address(&account_pubkey, &KMNO_MINT_ADDRESS);
    let (user_state, _user_state_bump) = Pubkey::find_program_address(
        &[
            BASE_SEED_USER_STATE,
            KMNO_FARM_STATE.as_ref(),
            account_pubkey.as_ref(),
        ],
        &KMNO_STAKING_PROGRAM,
    );

    let mut unstake_data = Sha256::new();
    unstake_data.update(b"global:withdraw_unstaked_deposits");
    let unstake_result = unstake_data.finalize();
    let unstake_first_8_bytes = &unstake_result[..8];

    let mut unstake_data = Vec::new();
    unstake_data.extend_from_slice(unstake_first_8_bytes);

    let stake_accounts = vec![
        AccountMeta::new(account_pubkey, true),
        AccountMeta::new(user_state, false),
        AccountMeta::new(KMNO_FARM_STATE, false),
        AccountMeta::new(kmno_ata, false),
        AccountMeta::new(KMNO_FARM_VAULT, false),
        AccountMeta::new_readonly(KMNO_FARM_VAULT_AUTHORITY, false),
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
    ];

    let unstake_instruction = unstake_instruction(account_pubkey,  user_state, user_amount);
    let withdraw_instruction =
        Instruction::new_with_bytes(KMNO_STAKING_PROGRAM, &unstake_data, stake_accounts);

    let instructions = vec![unstake_instruction, withdraw_instruction];

    return instructions;
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
