use errors::ActionError;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use solana_sdk::{
    instruction::AccountMeta, instruction::Instruction, message::Message, pubkey, pubkey::Pubkey,
    transaction::Transaction,
};
use spl_associated_token_account::get_associated_token_address;
use spl_token::ID as TOKEN_PROGRAM_ID;
use std::str::FromStr;
use znap::prelude::*;

mod errors;

const KMNO_MINT_ADDRESS: Pubkey = pubkey!("KMNo3nJsBXfcpJTVhZcXLW7RmTwTt4GVFE7suUBo9sS");
const KMNO_DECIMALS: u8 = 6;
const KMNO_STAKING_PROGRAM: Pubkey = pubkey!("FarmsPZpWu9i7Kky8tPN37rs2TpmMrAZrC7S7vJa91Hr");

#[collection]
pub mod kmno_staking {
    use super::*;

    pub fn stake(ctx: Context<StakeAction>) -> Result<ActionTransaction> {
        let account_pubkey = Pubkey::from_str(&ctx.payload.account)
            .or_else(|_| Err(Error::from(ActionError::InvalidAccountPublicKey)))?;

        let kmno_ata = get_associated_token_address(&account_pubkey, &KMNO_MINT_ADDRESS);

        let seeds: &[&[u8]] = &[&KMNO_MINT_ADDRESS.to_bytes()];
        let (vault_pubkey, _vault_bump) =
            Pubkey::find_program_address(seeds, &KMNO_STAKING_PROGRAM);

        let decimals_result = 10u32.pow(KMNO_DECIMALS as u32);
        let amount = (ctx.query.amount * (decimals_result as f32)) as u64;

        // Stake instruction

        let stake_args = StakeInstructionArgs { amount };
        let stake_serialized_args =
            bincode::serialize(&stake_args).expect("Error serializing args");

        let mut stake_hasher = Sha256::new();
        stake_hasher.update(b"global:stake");
        let stake_result = stake_hasher.finalize();
        let stake_first_8_bytes = &stake_result[..8];

        let mut stake_data = Vec::new();
        stake_data.extend_from_slice(stake_first_8_bytes);
        stake_data.extend_from_slice(&stake_serialized_args);

        let stake_accounts = vec![
            AccountMeta::new_readonly(account_pubkey, true),
            // AccountMeta::new(xstep_mint, false), ||| userState
            // AccountMeta::new(step_associated_token_address, false), ||| farmState
            AccountMeta::new(vault_pubkey, false),
            AccountMeta::new(kmno_ata, false),
            AccountMeta::new_readonly(KMNO_STAKING_PROGRAM, false),
            // AccountMeta::new_readonl(KMNO_STAKING_PROGRAM, false), ||| scopePrices
            AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
        ];

        let stake_instruction =
            Instruction::new_with_bytes(KMNO_STAKING_PROGRAM, &stake_data, stake_accounts);

        let instructions = vec![stake_instruction];
        let message = Message::new(&instructions, None);
        let transaction = Transaction::new_unsigned(message);

        Ok(ActionTransaction {
            transaction,
            message: Some("Stake successfully completed".to_string()),
        })
    }
}

#[derive(Action)]
#[action(
    icon = "https://raw.githubusercontent.com/leandrogavidia/files/main/kmno-staking.png",
    title = "Stake KMNO",
    description = "Stake your KMNO to boost points, vote on proposals, and earn rewards in Kamino Finance",
    label = "Stake",
    link = {
        label = "Stake",
        href = "/api/stake?amount={amount}",
        parameter = { label = "Amount", name = "amount"  }
    },
)]
#[query(amount: f32)]
pub struct StakeAction;

#[derive(Serialize, Deserialize)]
pub struct StakeInstructionArgs {
    pub amount: u64,
}
