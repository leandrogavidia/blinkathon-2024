use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use znap::prelude::*;

use crate::errors::ActionError;

pub async fn get_token_metadata(mint_address: &String, rpc: &String) -> Result<Value> {
    let client = Client::new();
    let base_url = rpc;

    let req = HeliusRequest {
        id: "text".to_string(),
        jsonrpc: "2.0".to_string(),
        method: "getAsset".to_string(),
        params: HeliusParams {
            id: mint_address.to_string(),
        },
    };

    let response = client
        .post(format!("{}", base_url))
        .header("Accept", "application/json")
        .json(&req)
        .send()
        .await
        .or_else(|_| Err(Error::from(ActionError::InternalServerError)))?;

    if response.status() == StatusCode::OK {
        return response
            .json::<Value>()
            .await
            .or_else(|_| Err(Error::from(ActionError::InvalidResponseBody)));
    }

    return Err(Error::from(ActionError::UnknownServerError));
}

#[derive(Debug, Serialize, Deserialize)]

struct HeliusParams {
    id: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct HeliusRequest {
    jsonrpc: String,
    id: String,
    method: String,
    params: HeliusParams,
}
