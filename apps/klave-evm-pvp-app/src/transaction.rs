use std::fmt::Display;

use serde::{Deserialize, Serialize};
use serde_json::to_string;
use crate::{user::{RoleType, User}, wallet::{self, Wallet}};
use alloy_primitives::{hex, U256};
use klave;

pub(crate) const TRANSACTION_TABLE: &str = "transactionTable";

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub enum PvPstate {
    Init,
    AwaitingSourceReceive,
    AwaitingSourceReceiveFinalized,
    AwaitingDestinationReceive,
    AwaitingDestinationReceiveFinalized,
    AwaitingDestinationSend,
    AwaitingDestinationSendFinalized,
    AwaitingSourceSend,
    AwaitingSourceSendFinalized,
    Complete,
    Cancelled,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Participant {
    pub network_name: String,
    pub address: String,
    pub amount: U256,    
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct NetworkTransaction {
    pub state: PvPstate,
    pub network_name: String,
    pub tx_hash: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PaymentVsPayment {
    pub source: Participant,
    pub destination: Participant,
    pub state_machine: PvPstate,
    pub network_transactions: Vec<NetworkTransaction>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Transaction {
    pub id: String,
    pub timestamp: String,
    pub payment_vs_payment: Option<PaymentVsPayment>,
    pub escrow_address: String,
}

impl Display for Transaction {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", match serde_json::to_string(self) {
            Ok(s) => s,
            Err(e) => {
                format!("ERROR: failed to serialize Transaction: {}", e)
            }
        })
    }
}

impl Transaction {
    pub fn new(pvp: &PaymentVsPayment) -> Result<Transaction, Box<dyn std::error::Error>> {
        let tx_id = klave::crypto::random::get_random_bytes(64).map(|x| hex::encode(x))?;
        Ok(Transaction {
            id: tx_id.clone(),
            timestamp: klave::context::get("trusted_time").unwrap_or("0".to_string()),
            payment_vs_payment: {
                let source_wallet = Wallet::load(&pvp.source.address)?;
                let destination_wallet = Wallet::load(&pvp.destination.address)?;
                for user in source_wallet.get_users() {
                    let mut u = User::load(user)?;
                    u.add_transaction(&tx_id, RoleType::Participant)?;
                }
                for user in destination_wallet.get_users() {
                    let mut u = User::load(&user)?;
                    u.add_transaction(&tx_id, RoleType::Participant)?;
                }
                let mut post_init_pvp = pvp.clone();
                post_init_pvp.state_machine = PvPstate::AwaitingSourceReceive;
                Some(post_init_pvp)
            },
            escrow_address: {
                let (secret_key, public_key) = match wallet::generate_keypair(None) {
                    Ok((sk, pk)) => (sk, pk),
                    Err(e) => {
                        klave::notifier::send_string(&format!("ERROR: failed to generate keypair: {}", e));
                        return Transaction::new(pvp);
                    }
                };
                let mut wallet = Wallet::new(&secret_key, &public_key);
                wallet.add_network(&pvp.source.network_name)?;    
                wallet.add_network(&pvp.destination.network_name)?;                

                let mut swift_user = match User::load(&klave::context::get("sender")?) {
                    Ok(u) => u,
                    Err(e) => {
                        return Err(format!("ERROR: failed to load swift user - {}", e).into());
                    }
                };
                swift_user.add_transaction(&tx_id, RoleType::Orchestrator)?;
                swift_user.add_wallet(&wallet.get_eth_address().to_string())?;

                wallet.add_user(&swift_user.id)?;
                wallet.add_transaction(&tx_id)?;
                wallet.get_eth_address().to_string()
            },
        })      
    }

    pub fn load(id: &str) -> Result<Transaction, Box<dyn std::error::Error>> {
        match klave::ledger::get_table(TRANSACTION_TABLE).get(&id) {        
            Ok(v) => {
                let tx: Transaction = match serde_json::from_slice(&v) {
                    Ok(w) => w,
                    Err(e) => {
                        klave::notifier::send_string(&format!("ERROR: failed to deserialize user: {}", e));
                        return Err(e.into());
                    }
                };
                Ok(tx)
            },
            Err(e) => Err(e.into())
        }
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let serialized_user = match to_string(&self) {
            Ok(s) => s,
            Err(e) => {
                klave::notifier::send_string(&format!("ERROR: failed to serialize user: {}", e));
                return Err(e.into());
            }
        };
        klave::ledger::get_table(TRANSACTION_TABLE).set(&self.id, &serialized_user.as_bytes())
    }

    pub fn process(&self) -> Result<(), Box<dyn std::error::Error>> {
        match &self.payment_vs_payment {
            Some(pvp) => {
                match &pvp.state_machine {
                    PvPstate::Init => {
                        //sign a message asking source to pay source amount into escrow account
                        
                        klave::notifier::send_string("Processing transaction");
                    }
                    PvPstate::AwaitingSourceSend => {
                        klave::notifier::send_string("Processing transaction");
                    }
                    PvPstate::AwaitingDestinationSend => {
                        klave::notifier::send_string("Processing transaction");
                    }
                    PvPstate::AwaitingSourceReceive => {
                        klave::notifier::send_string("Processing transaction");
                    }
                    PvPstate::AwaitingDestinationReceive => {
                        klave::notifier::send_string("Processing transaction");
                    }
                    PvPstate::Complete => {
                        klave::notifier::send_string("Transaction already processed");
                    }
                    PvPstate::Cancelled => {
                        klave::notifier::send_string("Transaction already processed");
                    }
                    _ => {
                        klave::notifier::send_string("Transaction already processed");
                    }
                }
            }
            None => {
                klave::notifier::send_string("Transaction not found");
            }
        }
        Ok(())
    }

}

