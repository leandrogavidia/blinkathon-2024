use errors::ActionError;
use instructions::{stake_instruction, withdraw_unstaked_deposits_instruction};
use solana_sdk::{message::Message, pubkey, pubkey::Pubkey, transaction::Transaction};
use std::str::FromStr;
use znap::prelude::*;
use spl_associated_token_account::instruction::create_associated_token_account_idempotent;
use spl_token::ID as TOKEN_PROGRAM_ID;

mod errors;
mod instructions;

const KMNO_MINT_ADDRESS: Pubkey = pubkey!("KMNo3nJsBXfcpJTVhZcXLW7RmTwTt4GVFE7suUBo9sS");

#[collection]
pub mod kmno_staking {
    use std::future::IntoFuture;

    use super::*;

    pub fn stake(ctx: Context<StakingAction>) -> Result<ActionTransaction> {
        let account_pubkey = Pubkey::from_str(&ctx.payload.account)
            .or_else(|_| Err(Error::from(ActionError::InvalidAccountPublicKey)))?;
        
        let method = ctx.query.method.clone();
        let amount = ctx.query.amount;
        let rpc = ctx.env.rpc_url.clone();

        let create_send_ata_instruction = create_associated_token_account_idempotent(
            &account_pubkey,
            &account_pubkey,
            &KMNO_MINT_ADDRESS,
            &TOKEN_PROGRAM_ID,
        );

        let mut instructions = vec![create_send_ata_instruction];

        let staking_instructions = if method == "stake" {
            stake_instruction(account_pubkey, amount, rpc).await
        } else {
            withdraw_unstaked_deposits_instruction(account_pubkey, amount)
        };

        instructions.extend_from_slice(&staking_instructions);

        let message = Message::new(&instructions, None);
        let transaction = Transaction::new_unsigned(message);

        Ok(ActionTransaction {
            transaction,
            message: Some(format!("{} successfully completed", method.to_uppercase())),
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
        href = "/api/staking?amount={amount}&method=stake",
        parameter = { label = "Amount", name = "amount"  }
    },
    link = {
        label = "Unstake",
        href = "/api/staking?amount={amount}&method=unstake",
        parameter = { label = "Amount", name = "amount"  }
    },
)]
#[query(amount: f32, method: String)]
pub struct StakingAction;
