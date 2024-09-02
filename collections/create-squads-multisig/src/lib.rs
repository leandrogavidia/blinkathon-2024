use errors::ActionError;
use solana_sdk::{
    instruction::AccountMeta, instruction::Instruction, message::Message, pubkey, pubkey::Pubkey,
    signature::Keypair, signer::Signer, system_program::ID as SYSTEM_PROGRAM_ID,
    transaction::Transaction,
};
use std::str::FromStr;
use structs::{CreateInstructionArgs, SquadsMetadata};
use znap::prelude::*;

mod errors;
mod structs;

const SQUADS_PROGRAM_ID: Pubkey = pubkey!("SMPLecH534NA9acpos4G6x7uf3LWbCAwZQE9e8ZekMu");
const CREATE_DISCRIMINANT: [u8; 8] = [24, 30, 200, 40, 5, 28, 7, 119];

#[collection]
pub mod create {
    use super::*;

    pub fn create(ctx: Context<CreateAction>) -> Result<ActionTransaction> {
        let account_pubkey = Pubkey::from_str(&ctx.payload.account)
            .or_else(|_| Err(Error::from(ActionError::InvalidAccountPublicKey)))?;

        let name = ctx.query.name.clone();
        let description = ctx.query.description.clone();

        if name.len() > 36 {
            return Err(Error::from(ActionError::InvalidNameLength));
        } else if description.len() > 64 {
            return Err(Error::from(ActionError::InvalidDescriptionLength));
        }

        let create_key = Keypair::new().pubkey();
        let threshold = 1;
        let members = vec![account_pubkey];
        let image = "".to_string();

        let meta_data = SquadsMetadata {
            name,
            description,
            image,
        };

        let meta = serde_json::to_string(&meta_data)
            .or_else(|_| Err(Error::from(ActionError::InvalidMetadata)))?;

        let (squads_key, _user_state_bump) = Pubkey::find_program_address(
            &[b"squad", create_key.as_ref(), b"multisig"],
            &SQUADS_PROGRAM_ID,
        );

        let args = CreateInstructionArgs {
            create_key,
            members,
            meta,
            threshold,
        };

        let accounts = vec![
            AccountMeta::new(squads_key, false),
            AccountMeta::new(account_pubkey, true),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
        ];

        let instruction = Instruction::new_with_borsh(
            SQUADS_PROGRAM_ID,
            &(CREATE_DISCRIMINANT, args),
            accounts,
        );

        let instructions = vec![instruction];
        let message = Message::new(&instructions, None);
        let transaction = Transaction::new_unsigned(message);

        Ok(ActionTransaction {
            transaction,
            message: Some("Multisig successfully created!".to_string()),
        })
    }

    fn get_create(ctx: Context<CreateAction>) -> Result<ActionMetadata> {
        let label = "Create!";
        let name = "{name}";
        let description = "{description}";

        let links = ActionLinks {
            actions: vec![LinkedAction {
                label: label.to_string(),
                href: format!("/api/create?name={}&description={}", name, description),
                parameters: vec![
                    LinkedActionParameter {
                        label: "Squad name (max 36 characters)".to_string(),
                        name: "name".to_string(),
                        required: true,
                    },
                    LinkedActionParameter {
                        label: "Squad description (max 64 characters)".to_string(),
                        name: "description".to_string(),
                        required: true,
                    },
                ],
            }],
        };

        Ok(ActionMetadata {
            title: "Create your multisig | SQUADS V3".to_string(),
            description: "The most secure and intuitive way to manage on-chain assets individually or together with your team".to_string(),
            icon: "https://raw.githubusercontent.com/leandrogavidia/files/main/create-squads-multisig.png".to_string(),
            label: label.to_string(),
            disabled: false,
            error: None,
            links: Some(links),
        })
    }
}

#[derive(Action)]
#[query(name: String, description: String)]
pub struct CreateAction;
