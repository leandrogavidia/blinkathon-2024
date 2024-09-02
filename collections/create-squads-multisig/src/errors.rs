use znap::prelude::*;

#[derive(ErrorCode)]
pub enum ActionError {
    #[error(msg = "Invalid account public key")]
    InvalidAccountPublicKey,
    #[error(msg = "Invalid Squads metadata")]
    InvalidMetadata,
    #[error(msg = "Invalid name length")]
    InvalidNameLength,
    #[error(msg = "Invalid description length")]
    InvalidDescriptionLength,
}