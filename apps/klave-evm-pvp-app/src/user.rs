use std::fmt::{self, Display, Formatter};

use alloy_primitives::hex;
use serde::{Deserialize, Serialize};
use serde_json::to_string;
use klave;
use crate::{transaction::Transaction, wallet::Wallet};

pub(crate) const USER_TABLE: &str = "userTable";

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum RoleType {
    None = 0,
    Orchestrator = 1,
    Participant = 2,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransactionRole {
    pub transaction_id: String,
    pub role: RoleType,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct User {    
    pub id: String,
    transactions: Vec<TransactionRole>,
    wallets: Vec<String>,    
}

impl Display for User {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match serde_json::to_string(self) {
            Ok(s) => s,
            Err(e) => {
                format!("ERROR: failed to serialize User: {}", e)
            }
        })
    }
}

impl User {
    pub fn new(id: &str) -> User {
        User {
            id: if id.is_empty() {
                klave::crypto::random::get_random_bytes(64).map(|x| hex::encode(x)).unwrap()
            }
            else {
                id.to_string()
            },
            transactions: Vec::new(),
            wallets: Vec::new(),
        }
    }

    pub fn load(id: &str) -> Result<User, Box<dyn std::error::Error>> {
        match klave::ledger::get_table(USER_TABLE).get(&id) {        
            Ok(v) => {
                let user: User = match serde_json::from_slice(&v) {
                    Ok(w) => w,
                    Err(e) => {
                        klave::notifier::send_string(&format!("ERROR: failed to deserialize user: {}", e));
                        return Err(e.into());
                    }
                };
                Ok(user)
            },
            Err(e) => Err(e.into())
        }
    }

    pub fn get(id: &str) -> User {
        match User::load(id) {
            Ok(nm) => nm,
            Err(_) => {
                User::new(id)
            }
        }
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let serialized_user = match to_string(&self) {
            Ok(s) => s,
            Err(e) => {                
                return Err(format!("ERROR: failed to serialize user: {}", e).into());
            }
        };
        klave::ledger::get_table(USER_TABLE).set(&self.id, &serialized_user.as_bytes())?;
        Ok(())
    }

    pub fn add_wallet(&mut self, address: &str) -> Result<(), Box<dyn std::error::Error>> {        
        //Check if wallet exists
        let mut found = false;
        for w in &self.wallets {
            if w == address {
                found = true;
                break;
            }
        }
        if found {
            return Err("wallet already exists for this user".into());
        }

        //Check if wallet is valid
        let mut existing_wallet = match Wallet::load(address) {
            Ok(w) => w,            
            Err(_) => {
                return Err("wallet does not exist".into());            
            }
        };
        existing_wallet.add_user(&self.id)?;

        self.wallets.push(address.to_string());
        self.save()?;
        Ok(())
    }

    pub fn add_transaction(&mut self, transaction_id: &str, role: RoleType) -> Result<(), Box<dyn std::error::Error>> {
        //Check if transaction exists
        let mut found = false;
        for r in &self.transactions {
            if r.role == role && r.transaction_id == transaction_id {
                found = true;
                break;
            }
        }
        if found {
            return Err("role already exists for this user".into());
        }

        //Check if wallet is valid
        match Transaction::load(transaction_id) {
            Ok(_) => {
                return Err("transaction does already exist".into());
            }
            Err(_) => ()
        }

        self.transactions.push(TransactionRole {
            role,
            transaction_id: transaction_id.to_string(),
        });
        self.save()?;
        Ok(())
    }

    pub fn get_wallets(&self) -> Vec<String> {
        self.wallets.clone()
    }

    pub fn get_transactions(&self) -> Vec<TransactionRole> {
        self.transactions.clone()
    }
}
