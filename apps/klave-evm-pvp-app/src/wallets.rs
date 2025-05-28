use serde::{Deserialize, Serialize};
use klave;
use std::fmt::{self, Display, Formatter};
use serde_json::to_string;

use crate::wallet::WALLET_TABLE;

#[derive(Serialize, Deserialize, Debug)]
pub struct WalletCreationInfo {
    pub address: String,
    pub timestamp: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Wallets {
    list: Vec<WalletCreationInfo>
}

impl Display for Wallets {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match serde_json::to_string(self) {
            Ok(s) => s,
            Err(e) => {
                format!("ERROR: failed to serialize Wallets: {}", e)
            }
        })
    }
}

impl Wallets {
    pub fn new() -> Wallets {
        Wallets {
            list: Vec::new()
        }
    }

    pub fn add_address(&mut self, address: &str) -> Result<(), Box<dyn std::error::Error>> {
        //Check if network exists
        let mut found = false;
        for w in &self.list {
            if w.address == address {
                found = true;
                break;
            }
        }
        if found {
            return Err("wallet already exists".into());
        }        

        let info = WalletCreationInfo {
            address: address.to_string(),
            timestamp: klave::context::get("trusted_time").unwrap_or("0".to_string())
        };

        self.list.push(info);
        if let Err(e) = self.save() {
            return Err(format!("failed to save wallet list: {}", e).into());
        }
        Ok(())
    }

    pub fn get_list_address(&self) -> &Vec<WalletCreationInfo> {
        &self.list
    }

    pub fn load() -> Result<Wallets, Box<dyn std::error::Error>> {
        match klave::ledger::get_table(WALLET_TABLE).get("ALL") {
            Ok(v) => {
                let wallet: Wallets = match serde_json::from_slice(&v) {
                    Ok(w) => w,
                    Err(e) => {
                        klave::notifier::send_string(&format!("ERROR: failed to deserialize wallet list: {}", e));
                        return Err(e.into());
                    }
                };
                Ok(wallet)
            },
            Err(e) => Err(e.into())
        }
    }

    pub fn get() -> Wallets {
        match Wallets::load() {
            Ok(nm) => nm,
            Err(_) => {
                Wallets::new()
            }
        }
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let serialized_wallet_list = match to_string(&self) {
            Ok(s) => s,
            Err(e) => {
                klave::notifier::send_string(&format!("ERROR: failed to serialize wallet list: {}", e));
                return Err(e.into());
            }
        };
        klave::ledger::get_table(WALLET_TABLE).set("ALL", &serialized_wallet_list.as_bytes())
    }
}
