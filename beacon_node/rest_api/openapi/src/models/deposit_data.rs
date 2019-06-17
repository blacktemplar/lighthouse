/*
 * Minimal Beacon Node API for Validator
 *
 * A minimal API specification for the beacon node, which enables a validator to connect and perform its obligations on the Ethereum 2.0 phase 0 beacon chain.
 *
 * The version of the OpenAPI document: 0.2.0
 * 
 * Generated by: https://openapi-generator.tech
 */

/// DepositData : The [`DepositData`](https://github.com/ethereum/eth2.0-specs/blob/master/specs/core/0_beacon-chain.md#depositdata) object from the Eth2.0 spec.

#[allow(unused_imports)]
use serde_json::Value;


#[derive(Debug, Serialize, Deserialize)]
pub struct DepositData {
    /// The validator's BLS public key, uniquely identifying them. _48-bytes, hex encoded with 0x prefix, case insensitive._
    #[serde(rename = "pubkey", skip_serializing_if = "Option::is_none")]
    pub pubkey: Option<String>,
    /// The withdrawal credentials.
    #[serde(rename = "withdrawal_credentials", skip_serializing_if = "Option::is_none")]
    pub withdrawal_credentials: Option<String>,
    /// Amount in Gwei.
    #[serde(rename = "amount", skip_serializing_if = "Option::is_none")]
    pub amount: Option<i32>,
    /// Container self-signature.
    #[serde(rename = "signature", skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
}

impl DepositData {
    /// The [`DepositData`](https://github.com/ethereum/eth2.0-specs/blob/master/specs/core/0_beacon-chain.md#depositdata) object from the Eth2.0 spec.
    pub fn new() -> DepositData {
        DepositData {
            pubkey: None,
            withdrawal_credentials: None,
            amount: None,
            signature: None,
        }
    }
}


