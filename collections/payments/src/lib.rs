use errors::ActionError;
use helius_api::get_token_metadata;
use jupiter_api::get_swap_instructions;
use solana_sdk::{message::Message, pubkey, pubkey::Pubkey, transaction::Transaction};
use spl_associated_token_account::get_associated_token_address;
use spl_associated_token_account::instruction::create_associated_token_account_idempotent;
use spl_token::ID as TOKEN_PROGRAM_ID;
use utils::format_pubkey;
use std::str::FromStr;
use znap::prelude::*;

mod errors;
mod field_instruction;
mod field_pubkey;
mod helius_api;
mod jupiter_api;
mod utils;

const SEND_MINT_ADDRESS: Pubkey = pubkey!("SENDdRQtYMWaQrBroBrJ2Q53fgVuq95CV9UPGEvpCxa");

#[collection]
pub mod payments { 
    use super::*;

    fn pay(ctx: Context<PayAction>) -> Result<ActionTransaction> {
        let account_pubkey = Pubkey::from_str(&ctx.payload.account)
            .or_else(|_| Err(Error::from(ActionError::InvalidAccountPublicKey)))?;
        let receiver_pubkey = Pubkey::from_str(&ctx.params.receiver)
            .or_else(|_| Err(Error::from(ActionError::InvalidReceiverPublicKey)))?;
        let token_mint = Pubkey::from_str(&ctx.params.token_mint)
            .or_else(|_| Err(Error::from(ActionError::InvalidTokenMintPublicKey)))?;

        let res = get_token_metadata(&ctx.params.token_mint, &ctx.env.rpc_url)
            .await
            .or_else(|_| Err(Error::from(ActionError::ErrorObtainingTokenMetadata)))?;

        let token_decimals = res["result"]["token_info"]["decimals"].as_u64().unwrap();

        let decimals_result = 10u32.pow(token_decimals as u32);
        let amount = (ctx.query.amount * (decimals_result as f32)) as u64;

        let receiver_send_ata_address = get_associated_token_address(&receiver_pubkey, &SEND_MINT_ADDRESS);

        let create_send_ata_instruction = create_associated_token_account_idempotent(
            &account_pubkey,
            &account_pubkey,
            &SEND_MINT_ADDRESS,
            &TOKEN_PROGRAM_ID,
        );

        let swap_instructions = get_swap_instructions(
            &account_pubkey.to_string(),
            &receiver_send_ata_address.to_string(),
            &token_mint.to_string(),
            &SEND_MINT_ADDRESS.to_string(),
            amount,
        )
        .await
        .or_else(|_| Err(Error::from(ActionError::ErrorObtainingSwapInstructions)))?;

        let token_ledger_instruction = swap_instructions.token_ledger_instruction;
        let swap_compute_budget_instructions = swap_instructions.compute_budget_instructions;
        let setup_instructions = swap_instructions.setup_instructions;
        let swap_instruction = swap_instructions.swap_instruction;
        let cleanup_instruction = swap_instructions.cleanup_instruction;

        let mut instructions = vec![create_send_ata_instruction];

        if let Some(instruction) = token_ledger_instruction {
            instructions.push(instruction);
        }

        instructions.extend_from_slice(&swap_compute_budget_instructions);
        instructions.extend_from_slice(&setup_instructions);
        instructions.push(swap_instruction);

        if let Some(instruction) = cleanup_instruction {
            instructions.push(instruction);
        }

        let message = Message::new(&instructions, None);
        let transaction = Transaction::new_unsigned(message);

        Ok(ActionTransaction {
            transaction,
            message: Some("Payment successfully sent".to_string()),
        })
    }

    fn get_pay(ctx: Context<PayAction>) -> Result<ActionMetadata> {
        let token_mint = &ctx.params.token_mint;
        let receiver_address = &ctx.params.receiver;

        let res = get_token_metadata(&token_mint, &ctx.env.rpc_url)
            .await
            .or_else(|_| Err(Error::from(ActionError::ErrorObtainingTokenMetadata)))?;

        let token_symbol = res["result"]["token_info"]["symbol"].as_u64().unwrap();

        let label = "Send payment!";
        let description = format!(
            "Pay in {} and {} receives in SEND",
            token_symbol, format_pubkey(&receiver_address.to_string(), 10)
        );
        let links = ActionLinks {
            actions: vec![LinkedAction {
                label: "Send payment!".to_string(),
                href: "/api/pay/{{params.token_mint}}/{{params.receiver}}?amount=amount".to_string(),
                parameters: vec![LinkedActionParameter {
                    label: "Amount".to_string(),
                    name: "amount".to_string(),
                    required: true,
                }],
            }],
        };

        Ok(ActionMetadata {
            title: "Pay with SEND using any Solana token".to_string(),
            description: description.to_string(),
            icon: "".to_string(),
            label: label.to_string(),
            disabled: false,
            error: None,
            links: Some(links),
        })
    }
}

#[derive(Action)]
#[query(amount: f32)]
#[params(token_mint: String, receiver: String)]
pub struct PayAction;

// #[derive(Action)]
// #[action(
//     icon = "",
//     title = "Pay with SEND using any Solana token",
//     description = "Pay with X and 1234567890 receives in SEND",
//     label = "Send payment!",
//     link = {
//         label = "Send payment!",
//         href = "/api/pay/{{params.token_mint}}/{{params.receiver}}?amount=amount",
//         parameter = { label = "Amount", name = "amount" }
//     }
// )]
// #[query(amount: f32)]
// #[params(token_mint: String, receiver: String)]
// pub struct PayAction;
