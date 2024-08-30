use errors::ActionError;
use instructions::{stake_instruction, unstake_instruction};
use solana_sdk::{message::Message, pubkey::Pubkey, transaction::Transaction};
use std::str::FromStr;
use znap::prelude::*;

mod errors;
mod instructions;

const KMNO_DECIMALS: u8 = 6;

#[collection]
pub mod kmno_staking {

    use super::*;

    pub fn stake(ctx: Context<StakingAction>) -> Result<ActionTransaction> {
        let account_pubkey = Pubkey::from_str(&ctx.payload.account)
            .or_else(|_| Err(Error::from(ActionError::InvalidAccountPublicKey)))?;

        let decimals_result = 10u32.pow(KMNO_DECIMALS as u32);
        let amount = (ctx.query.amount * (decimals_result as f32)) as u64;
        let method = ctx.query.method.clone();

        let instruction = if method == "stake" {
            stake_instruction(account_pubkey, amount)
        } else {
            unstake_instruction(account_pubkey, amount)
        };

        let message = Message::new(&[instruction], None);
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
        href = "/api/stake?amount={amount}&method=stake",
        parameter = { label = "Amount", name = "amount"  }
    },
    link = {
        label = "Unstake",
        href = "/api/stake?amount={amount}&method=unstake",
        parameter = { label = "Amount", name = "amount"  }
    },
)]
#[query(amount: f32, method: String)]
pub struct StakingAction;
