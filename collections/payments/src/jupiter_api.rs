use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use solana_sdk::{instruction::Instruction, pubkey::Pubkey};
use znap::prelude::*;

use crate::errors::ActionError;
use crate::field_instruction;
use crate::field_pubkey;

pub async fn get_swap_instructions(
    account_pubkey: &String,
    destination_token_account: &String,
    input_mint_address: &String,
    output_mint_address: &String,
    amount: u64,
) -> Result<SwapInstructions> {
    let client = Client::new();
    let base_url = "https://quote-api.jup.ag/v6";

    let max_accounts = "50";

    let quote_response = client
        .get(format!(
            "{}/quote?inputMint={}&outputMint={}&amount={}&maxAccounts={}",
            base_url, input_mint_address, output_mint_address, amount, max_accounts
        ))
        .send()
        .await
        .or_else(|_| Err(Error::from(ActionError::InternalServerError)))?
        .json::<QuoteResponse>()
        .await
        .or_else(|_| Err(Error::from(ActionError::QuoteNotFound)))?;

    let swap_request = SwapRequest {
        quote_response,
        user_public_key: account_pubkey.to_string(),
        destination_token_account: destination_token_account.to_string(),
    };

    let swap_instructions = client
        .post(format!("{}/swap-instructions", base_url))
        .header("Accept", "application/json")
        .json(&swap_request)
        .send()
        .await
        .or_else(|_| Err(Error::from(ActionError::InternalServerError)))?;

    if swap_instructions.status() == StatusCode::OK {
        return swap_instructions
            .json::<SwapInstructions>()
            .await
            .or_else(|_| Err(Error::from(ActionError::InvalidResponseBody)));
    }

    return Err(Error::from(ActionError::UnknownServerError));
}

#[derive(Debug, Serialize, Deserialize)]
struct SplTokenInfo {
    decimals: u8,
    supply: u64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SwapInfo {
    amm_key: String,
    label: String,
    input_mint: String,
    output_mint: String,
    in_amount: String,
    out_amount: String,
    fee_amount: String,
    fee_mint: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Route {
    swap_info: SwapInfo,
    percent: u32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct QuoteResponse {
    input_mint: String,
    in_amount: String,
    output_mint: String,
    out_amount: String,
    other_amount_threshold: String,
    swap_mode: String,
    slippage_bps: u32,
    platform_fee: Option<u32>,
    price_impact_pct: String,
    route_plan: Vec<Route>,
    context_slot: u64,
    time_taken: f64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SwapRequest {
    quote_response: QuoteResponse,
    user_public_key: String,
    destination_token_account: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SwapInstructions {
    #[serde(with = "field_instruction::option_instruction")]
    pub token_ledger_instruction: Option<Instruction>,
    #[serde(with = "field_instruction::vec_instruction")]
    pub compute_budget_instructions: Vec<Instruction>,
    #[serde(with = "field_instruction::vec_instruction")]
    pub setup_instructions: Vec<Instruction>,
    #[serde(with = "field_instruction::instruction")]
    pub swap_instruction: Instruction,
    #[serde(with = "field_instruction::option_instruction")]
    pub cleanup_instruction: Option<Instruction>,
    #[serde(with = "field_pubkey::vec")]
    pub address_lookup_table_addresses: Vec<Pubkey>,
    pub prioritization_fee_lamports: u64,
}
