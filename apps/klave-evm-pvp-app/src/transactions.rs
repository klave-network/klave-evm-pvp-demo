use std::fmt::Display;
use serde::{Deserialize, Serialize};
use serde_json::to_string;
use crate::transaction::{Transaction, TRANSACTION_TABLE};
use klave;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Transactions {
    list: Vec<String>
}

impl Display for Transactions {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", match serde_json::to_string(self) {
            Ok(s) => s,
            Err(e) => {
                format!("ERROR: failed to serialize Transactions: {}", e)
            }
        })
    }
}

impl Transactions {
    fn new() -> Transactions {
        Transactions {
            list: Vec::new(),
        }
    }

    pub fn load() -> Result<Transactions, Box<dyn std::error::Error>> {
        match klave::ledger::get_table(TRANSACTION_TABLE).get("ALL") {
            Ok(v) => {
                let wallet: Transactions = match serde_json::from_slice(&v) {
                    Ok(w) => w,
                    Err(e) => {
                        klave::notifier::send_string(&format!("ERROR: failed to parse transaction list: {}", e));
                        return Err(e.into());
                    }
                };
                Ok(wallet)
            },
            Err(e) => {
                Err(e.into())
            }
        }
    }

    pub fn get() -> Transactions {
        match Transactions::load() {
            Ok(nm) => nm,
            Err(_) => {
                Transactions::new()
            }
        }
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let serialized_txs = match to_string(&self) {
            Ok(s) => s,
            Err(e) => {
                klave::notifier::send_string(&format!("ERROR: failed to serialize transaction list: {}", e));
                return Err(e.into());
            }
        };
        match klave::ledger::get_table(TRANSACTION_TABLE).set("ALL", &serialized_txs.as_bytes()) {
            Ok(_) => Ok(()),
            Err(e) => Err(e.into())
        }
    }

    pub fn add_transaction(&mut self, tx_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        //Check if transaction exists
        let mut found = false;
        for n in &self.list {
            if n == &tx_id {
                found = true;
                break;
            }
        }
        if found {
            klave::notifier::send_string(&format!("ERROR: transaction {} already exists", tx_id));
            return Err("transaction already exists".into());
        }        

        self.list.push(tx_id.to_string());

        match self.save() {
            Ok(_) => {},
            Err(e) => {
                klave::notifier::send_string(&format!("ERROR: failed to save transaction list: {}", e));
                return Err(e.into());
            }
        }

        klave::notifier::send_string(&format!("transaction {} added", tx_id));
        Ok(())
    }

    pub fn get_transaction(&self, name: &str) -> Option<Transaction> {
        for transaction in &self.list {
            if transaction == name {
                let transaction = match Transaction::load(transaction) {
                    Ok(n) => n,
                    Err(e) => {
                        klave::notifier::send_string(&format!("ERROR: failed to load transaction: {}", e));
                        return None;
                    }
                };
                return Some(transaction);
            }
        }
        None
    }

    pub fn get_transactions(&self) -> &Vec<String> {
        &self.list
    }    
}
