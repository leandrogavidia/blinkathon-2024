use znap::prelude::*;

#[derive(ErrorCode)]
pub enum ActionError {
    #[error(msg = "Invalid account public key")]
    InvalidAccountPublicKey,
}