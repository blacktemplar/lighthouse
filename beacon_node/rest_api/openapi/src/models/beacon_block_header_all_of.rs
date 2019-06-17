/*
 * Minimal Beacon Node API for Validator
 *
 * A minimal API specification for the beacon node, which enables a validator to connect and perform its obligations on the Ethereum 2.0 phase 0 beacon chain.
 *
 * The version of the OpenAPI document: 0.2.0
 * 
 * Generated by: https://openapi-generator.tech
 */


#[allow(unused_imports)]
use serde_json::Value;


#[derive(Debug, Serialize, Deserialize)]
pub struct BeaconBlockHeaderAllOf {
    /// The tree hash merkle root of the `BeaconBlockBody` for the `BeaconBlock`
    #[serde(rename = "body_root", skip_serializing_if = "Option::is_none")]
    pub body_root: Option<String>,
}

impl BeaconBlockHeaderAllOf {
    pub fn new() -> BeaconBlockHeaderAllOf {
        BeaconBlockHeaderAllOf {
            body_root: None,
        }
    }
}


