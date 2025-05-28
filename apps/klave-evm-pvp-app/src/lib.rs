#[allow(warnings)]
mod bindings;

use std::str::FromStr;

use alloy_consensus::TxEip1559;
use alloy_primitives::{hex, Address, Bytes, TxKind, U256};
use alloy_rpc_types_eth::AccessList;
use alloy_signer::k256::SecretKey;
use bindings::Guest;
use klave;
use serde_json::Value;
use crate::klave_networks::{networks::Networks, network::Network};
use solidity::{burnCall, mintCall};

use transactions::Transactions;
use transaction::{NetworkTransaction, Participant, PaymentVsPayment, PvPstate, Transaction};
use wallet::Wallet;
use users::Users;
use user::User;
use alloy_sol_types::SolCall;


pub mod klave_networks;
pub mod wallet;
pub mod wallets;
pub mod transactions;
pub mod transaction;
pub mod users;
pub mod user;
pub mod solidity; 
pub mod eth;
pub mod web3;

/// Custom function to use the import for random byte generation.
///
/// We do this is because "js" feature is incompatible with the component model
/// if you ever got the __wbindgen_placeholder__ error when trying to use the `js` feature
/// of getrandom,
fn imported_random(dest: &mut [u8]) -> Result<(), getrandom::Error> {
    // iterate over the length of the destination buffer and fill it with random bytes
    let random_bytes = klave::crypto::random::get_random_bytes(dest.len().try_into().unwrap()).unwrap();
    dest.copy_from_slice(&random_bytes);

    Ok(())
}

getrandom::register_custom_getrandom!(imported_random);

struct Component;
impl Guest for Component {

    fn register_routes(){
        klave::router::add_user_transaction("network_add");
        klave::router::add_user_transaction("network_remove");
        klave::router::add_user_transaction("network_set_chain_id");
        klave::router::add_user_transaction("network_set_gas_price");
        klave::router::add_user_query("networks_all");

        klave::router::add_user_transaction("wallet_add");
        klave::router::add_user_transaction("wallet_add_network");
        klave::router::add_user_transaction("wallet_lock");
        klave::router::add_user_transaction("wallet_unlock");
        klave::router::add_user_query("wallet_address");
        klave::router::add_user_query("wallet_secret_key");
        klave::router::add_user_query("wallet_public_key");
        klave::router::add_user_query("wallet_balance");
        klave::router::add_user_query("wallet_networks");
        klave::router::add_user_query("wallet_transfer");
        klave::router::add_user_query("wallet_deploy_contract");
        klave::router::add_user_query("wallet_call_contract");        
        klave::router::add_user_query("wallets_all_for_user");
        klave::router::add_user_query("wallets_all");

        klave::router::add_user_transaction("user_add");
        klave::router::add_user_query("user_get");
        klave::router::add_user_query("users_all");
        klave::router::add_user_transaction("user_add_wallet");
        klave::router::add_user_transaction("transaction_add");
        klave::router::add_user_query("transaction_get");
        klave::router::add_user_transaction("transaction_commit");
        klave::router::add_user_transaction("transaction_apply");
        klave::router::add_user_query("transactions_all_for_user");    

        klave::router::add_user_query(&String::from("eth_block_number"));
        klave::router::add_user_query(&String::from("eth_get_block_by_number"));
        klave::router::add_user_query(&String::from("eth_gas_price"));
        klave::router::add_user_query(&String::from("eth_estimate_gas"));
        klave::router::add_user_query(&String::from("eth_call_contract"));
        klave::router::add_user_query(&String::from("eth_protocol_version"));
        klave::router::add_user_query(&String::from("eth_chain_id"));
        klave::router::add_user_query(&String::from("eth_get_transaction_by_hash"));
        klave::router::add_user_query(&String::from("eth_get_transaction_receipt")); 
        klave::router::add_user_query(&String::from("eth_get_transaction_count"));   

        klave::router::add_user_query(&String::from("web3_client_version"));
        klave::router::add_user_query(&String::from("web3_sha3"));
        klave::router::add_user_query(&String::from("net_version"));

        klave::router::add_user_query(&String::from("get_sender"));
        klave::router::add_user_query(&String::from("get_trusted_time"));
    }
    
    fn network_add(cmd: String){
        let Ok(v) = serde_json::from_str::<Value>(&cmd) else {
            klave::notifier::send_string(&format!("ERROR: failed to parse '{}' as json", cmd));
            return
        };

        let network_name = match v["network_name"].as_str() {
            Some(c) => c,
            None => {
                klave::notifier::send_string(&format!("ERROR: network not found"));
                return;
            }
        };
        let chain_id = v["chain_id"].as_u64();
        let rpc_url = match v["rpc_url"].as_str() {
            Some(c) => c,
            None => {
                klave::notifier::send_string(&format!("ERROR: rpc_url not found"));
                return;
            }
        };
        let gas_price = v["gas_price"].as_u64();
        let credentials = v["credentials"].as_str();
        let network = Network::new(network_name, chain_id, rpc_url, gas_price, credentials);        

        let mut nm = Networks::get();
        match nm.add_network(&network) {
            Ok(_) => {
                klave::notifier::send_string(&format!("network '{}' added", network_name));
            },
            Err(e) => {
                klave::notifier::send_string(&format!("ERROR: failed to add network '{}': {}", network_name, e));
            }
        }
    }

    fn network_remove(cmd: String){
        let Ok(v) = serde_json::from_str::<Value>(&cmd) else {
            klave::notifier::send_string(&format!("ERROR: failed to parse '{}' as json", cmd));
            return
        };

        let mut nm = match Networks::load() {
            Ok(nm) => nm,
            Err(e) => {
                klave::notifier::send_string(&format!("ERROR: failed to load network manager: {}. Create one first.", e));                
                return
            }
        };

        let network_name = match v["network_name"].as_str() {
            Some(c) => c,
            None => {
                klave::notifier::send_string(&format!("ERROR: network_name not found"));
                return;
            }
        };
        match nm.remove_network(network_name) {
            Ok(_) => {
                klave::notifier::send_string(&format!("network '{}' added", network_name));
            },
            Err(e) => {
                klave::notifier::send_string(&format!("ERROR: failed to add network '{}': {}", network_name, e));
            }
        }
    }

    fn network_set_chain_id(cmd: String){
        let Ok(v) = serde_json::from_str::<Value>(&cmd) else {
            klave::notifier::send_string(&format!("ERROR: failed to parse '{}' as json", cmd));
            return
        };
    
        let nm = match Networks::load() {
            Ok(nm) => nm,
            Err(e) => {
                klave::notifier::send_string(&format!("ERROR: failed to load network manager: {}. Create one first.", e));                
                return
            }
        };

        let network_name = match v["network_name"].as_str() {
            Some(c) => c,
            None => {
                klave::notifier::send_string(&format!("ERROR: network_name not found"));
                return;
            }
        };
        let chain_id = match v["chain_id"].as_u64() {
            Some(c) => c,
            None => {
                klave::notifier::send_string(&format!("ERROR: chain_id not found"));
                return;
            }
        };
        match nm.update_chain_id(network_name, chain_id) {
            Ok(_) => {
                klave::notifier::send_string(&format!("chain_id '{}' set as current", chain_id));
            },
            Err(e) => {
                klave::notifier::send_string(&format!("ERROR: failed to set chain_id '{}': {}", chain_id, e));
            }
        }
    }

    fn network_set_gas_price(cmd: String){
        let Ok(v) = serde_json::from_str::<Value>(&cmd) else {
            klave::notifier::send_string(&format!("ERROR: failed to parse '{}' as json", cmd));
            return
        };
    
        let nm = match Networks::load() {
            Ok(nm) => nm,
            Err(e) => {
                klave::notifier::send_string(&format!("ERROR: failed to load network manager: {}. Create one first.", e));                
                return
            }
        };

        let network_name = match v["network_name"].as_str() {
            Some(c) => c,
            None => {
                klave::notifier::send_string(&format!("ERROR: network_name not found"));
                return;
            }
        };
        let gas_price = match v["gas_price"].as_u64() {
            Some(c) => c,
            None => {
                klave::notifier::send_string(&format!("ERROR: gas_price not found"));
                return;
            }
        };
        match nm.update_gas_price(network_name, gas_price) {
            Ok(_) => {
                klave::notifier::send_string(&format!("gas_price '{}' set as current", gas_price));
            },
            Err(e) => {
                klave::notifier::send_string(&format!("ERROR: failed to set gas_price '{}': {}", gas_price, e));
            }
        }
    }

    fn networks_all(_cmd: String){
        let nm = match Networks::load() {
            Ok(nm) => nm,
            Err(e) => {
                klave::notifier::send_string(&format!("ERROR: failed to load network manager: {}. Create one first.", e));                
                return
            }
        };

        let mut networks: Vec<String> = Vec::<String>::new();
        for network_name in nm.get_networks() {
            let network = match Network::load(network_name) {
                Ok(n) => n.to_string(),
                Err(e) => {
                    klave::notifier::send_string(&format!("ERROR: failed to load network '{}': {}", network_name, e));
                    return;
                }
            };
            networks.push(network);
        }

        klave::notifier::send_string(&format!("{}", &serde_json::to_string(&networks).unwrap()));
    }

    fn wallet_add(cmd: String){
        let Ok(v) = serde_json::from_str::<Value>(&cmd) else {
            klave::notifier::send_string(&format!("ERROR: failed to parse '{}' as json", cmd));
            return;
        };
        
        let (secret_key, public_key) = match wallet::generate_keypair(v["secret_key"].as_str()) {
            Ok((s, p)) => (s, p),
            Err(e) => {
                klave::notifier::send_string(&format!("ERROR: failed to generate keypair: {}", e));
                return;
            }
        };
        let wallet = Wallet::new(&secret_key, &public_key);        
        let eth_address = wallet.get_eth_address();

        let mut wallets = wallets::Wallets::get();
        let mut found = false;
        for w in wallets.get_list_address() {
            if w.address == eth_address {
                found = true;
                break;
            }
        }

        if found {
            klave::notifier::send_string(&format!("ERROR: wallet {} already exists", eth_address));
            return;
        }

        match wallet.save() {
            Ok(_) => (),
            Err(e) => {
                klave::notifier::send_string(&format!("ERROR: failed to save wallet: {}", e));
                return;
            }
        };


        match wallets.add_address(&eth_address) {
            Ok(_) => {
                klave::notifier::send_string(&format!("wallet '{}' added", eth_address));
            },
            Err(e) => {
                klave::notifier::send_string(&format!("ERROR: failed to add wallet '{}': {}", eth_address, e));
            }
        }
    }

    fn wallet_add_network(cmd: String){
        let Ok(v) = serde_json::from_str::<Value>(&cmd) else {
            klave::notifier::send_string(&format!("ERROR: failed to parse '{}' as json", cmd));
            return;
        };

        let network_name = match v["network_name"].as_str() {
            Some(c) => c,
            None => {
                klave::notifier::send_string(&format!("ERROR: network_name not found"));
                return;
            }
        };

        let eth_address = match v["eth_address"].as_str() {
            Some(c) => c,
            None => {
                klave::notifier::send_string(&format!("ERROR: eth_address not found"));
                return;
            }
        };

        let mut wallet = match Wallet::load(eth_address) {
            Ok(w) => w,
            Err(e) => {
                klave::notifier::send_string(&format!("ERROR: failed to load wallet: {}", e));
                return;
            }
        };

        match wallet.add_network(network_name) {
            Ok(_) => klave::notifier::send_string(&format!("new network {} added to wallet", network_name)),
            Err(e) => klave::notifier::send_string(&format!("ERROR: failed to add network {}: {}", network_name, e))
        };        
    }

    fn wallet_lock(cmd: String){
        let Ok(v) = serde_json::from_str::<Value>(&cmd) else {
            klave::notifier::send_string(&format!("ERROR: failed to parse '{}' as json", cmd));
            return;
        };

        let eth_address = match v["eth_address"].as_str() {
            Some(c) => c,
            None => {
                klave::notifier::send_string(&format!("ERROR: eth_address not found"));
                return;
            }
        };

        let mut wallet = match Wallet::load(eth_address) {
            Ok(w) => w,
            Err(e) => {
                klave::notifier::send_string(&format!("ERROR: failed to load wallet: {}", e));
                return;
            }
        };

        let value = match v["value"].as_str() {
            Some(c) => {
                match U256::from_str_radix(c.trim_start_matches("0x"), 16) {
                    Ok(v) => v,
                    Err(e) => {
                        klave::notifier::send_string(&format!("ERROR: failed to parse value: {}", e));
                        return;
                    }
                }
            },
            None => {
                klave::notifier::send_string(&format!("ERROR: value not found"));
                return;
            }
        };

        let balance = match v["balance"].as_str() {
            Some(c) => {
                match U256::from_str_radix(c.trim_start_matches("0x"), 16) {
                    Ok(v) => v,
                    Err(e) => {
                        klave::notifier::send_string(&format!("ERROR: failed to parse balance: {}", e));
                        return;
                    }
                }
            },
            None => {
                klave::notifier::send_string(&format!("ERROR: balance not found"));
                return;
            }
        };

        let network_name = match v["network_name"].as_str() {
            Some(c) => c,
            None => {
                klave::notifier::send_string(&format!("ERROR: network_name not found"));
                return;
            }
        };

        match wallet.lock(network_name, value, balance) {
            Ok(proof) => klave::notifier::send_string(&format!("locked {} for wallet. here's the proof: {}", value, proof)),
            Err(e) => klave::notifier::send_string(&format!("ERROR: failed to lock {} for wallet: {}", value, e))
        };        
    }

    fn wallet_unlock(cmd: String) {
        let Ok(v) = serde_json::from_str::<Value>(&cmd) else {
            klave::notifier::send_string(&format!("ERROR: failed to parse '{}' as json", cmd));
            return;
        };

        let eth_address = match v["eth_address"].as_str() {
            Some(c) => c,
            None => {
                klave::notifier::send_string(&format!("ERROR: eth_address not found"));
                return;
            }
        };

        let mut wallet = match Wallet::load(eth_address) {
            Ok(w) => w,
            Err(e) => {
                klave::notifier::send_string(&format!("ERROR: failed to load wallet: {}", e));
                return;
            }
        };

        let value = match v["value"].as_str() {
            Some(c) => {
                match U256::from_str_radix(c.trim_start_matches("0x"), 16) {
                    Ok(v) => v,
                    Err(e) => {
                        klave::notifier::send_string(&format!("ERROR: failed to parse value: {}", e));
                        return;
                    }
                }
            },
            None => {
                klave::notifier::send_string(&format!("ERROR: value not found"));
                return;
            }
        };

        let network_name = match v["network_name"].as_str() {
            Some(c) => c,
            None => {
                klave::notifier::send_string(&format!("ERROR: network_name not found"));
                return;
            }
        };

        match wallet.unlock(network_name, value) {
            Ok(proof) => klave::notifier::send_string(&format!("unlocked {} for wallet. here's the proof: {}", value, proof)),
            Err(e) => klave::notifier::send_string(&format!("ERROR: failed to unlock {} for wallet: {}", value, e))
        };        

    }

    fn wallet_address(cmd: String){
        let Ok(v) = serde_json::from_str::<Value>(&cmd) else {
            klave::notifier::send_string(&format!("ERROR: failed to parse '{}' as json", cmd));
            return;
        };

        let eth_address = match v["eth_address"].as_str() {
            Some(c) => c,
            None => {
                klave::notifier::send_string(&format!("ERROR: eth_address not found"));
                return;
            }
        };

        let wallet = match Wallet::load(eth_address) {
            Ok(w) => w,
            Err(e) => {
                klave::notifier::send_string(&format!("ERROR: failed to load wallet: {}", e));
                return;
            }
        };

        klave::notifier::send_string(&wallet.get_eth_address());
    }

    fn wallet_secret_key(cmd: String){
        let Ok(v) = serde_json::from_str::<Value>(&cmd) else {
            klave::notifier::send_string(&format!("ERROR: failed to parse '{}' as json", cmd));
            return;
        };

        let eth_address = match v["eth_address"].as_str() {
            Some(c) => c,
            None => {
                klave::notifier::send_string(&format!("ERROR: eth_address not found"));
                return;
            }
        };

        let wallet = match Wallet::load(eth_address) {
            Ok(w) => w,
            Err(e) => {
                klave::notifier::send_string(&format!("ERROR: failed to load wallet: {}", e));
                return;
            }
        };

        klave::notifier::send_string(&wallet.get_secret_key());
    }

    fn wallet_public_key(cmd: String){
        let Ok(v) = serde_json::from_str::<Value>(&cmd) else {
            klave::notifier::send_string(&format!("ERROR: failed to parse '{}' as json", cmd));
            return;
        };

        let eth_address = match v["eth_address"].as_str() {
            Some(c) => c,
            None => {
                klave::notifier::send_string(&format!("ERROR: eth_address not found"));
                return;
            }
        };

        let wallet = match Wallet::load(eth_address) {
            Ok(w) => w,
            Err(e) => {
                klave::notifier::send_string(&format!("ERROR: failed to load wallet: {}", e));
                return;
            }
        };

        klave::notifier::send_string(&wallet.get_public_key());
    }

    fn wallet_networks(cmd: String){
        let Ok(v) = serde_json::from_str::<Value>(&cmd) else {
            klave::notifier::send_string(&format!("ERROR: failed to parse '{}' as json", cmd));
            return;
        };

        let eth_address = match v["eth_address"].as_str() {
            Some(c) => c,
            None => {
                klave::notifier::send_string(&format!("ERROR: eth_address not found"));
                return;
            }
        };

        let wallet = match Wallet::load(eth_address) {
            Ok(w) => w,
            Err(e) => {
                klave::notifier::send_string(&format!("ERROR: failed to load wallet: {}", e));
                return;
            }
        };

        let local_networks = wallet.get_networks();
        let local_networks_str: Vec<String> = local_networks.iter().map(|network| format!("{}", serde_json::to_string(&network).unwrap())).collect();
        klave::notifier::send_string(&format!("{}", serde_json::to_string(&local_networks_str).unwrap()));
    }

    fn wallet_transfer(cmd: String){
        let Ok(v) = serde_json::from_str::<Value>(&cmd) else {
            klave::notifier::send_string(&format!("ERROR: failed to parse '{}' as json", cmd));
            return;
        };    
    
        let chain_id = match v["chainId"].as_u64() {
            Some(c) => c,
            None => {
                klave::notifier::send_string(&format!("ERROR: chainId not found"));
                return;
            }
        };
        let nonce = match v["nonce"].as_u64() {
            Some(n) => n,
            None => {
                klave::notifier::send_string(&format!("ERROR: nonce not found"));
                return;
            }
        };
        let gas_limit = match v["gasLimit"].as_u64(){
            Some(g) => g,
            None => {
                klave::notifier::send_string(&format!("ERROR: gasLimit not found"));
                return;
            }
        };
        let to_str = match v["to"].as_str(){
            Some(t) => t,
            None => {
                klave::notifier::send_string(&format!("ERROR: to not found"));
                return;
            }
        };
        let to = match Address::from_str(&to_str) {
            Ok(a) => a,
            Err(e) => {
                klave::notifier::send_string(&format!("ERROR: failed to parse address: {}", e));
                return;
            }
        };
        let value = match v["value"].as_str() {
            Some(c) => {
                match U256::from_str_radix(c.trim_start_matches("0x"), 16) {
                    Ok(v) => v,
                    Err(e) => {
                        klave::notifier::send_string(&format!("ERROR: failed to parse value: {}", e));
                        return;
                    }
                }
            },
            None => {
                klave::notifier::send_string(&format!("ERROR: value not found"));
                return;
            }
        };
        let max_fee_per_gas = match v["maxFeePerGas"].as_u64() {
            Some(m) => m,
            None => {
                klave::notifier::send_string(&format!("ERROR: maxFeePerGas not found"));
                return;
            }
        };
        let max_priority_fee_per_gas = match v["maxPriorityFeePerGas"].as_u64() {
            Some(m) => m,
            None => {
                klave::notifier::send_string(&format!("ERROR: maxPriorityFeePerGas not found"));
                return;
            }
        };
    
        let tx = TxEip1559 {
            chain_id: chain_id,
            nonce: nonce,
            gas_limit: gas_limit,
            to: to.into(),
            value: value,
            input: Bytes::new(),
            max_fee_per_gas: max_fee_per_gas as u128,
            max_priority_fee_per_gas: max_priority_fee_per_gas as u128,
            access_list: AccessList::default(),
        };

        let network_name = match v["network_name"].as_str() {
            Some(c) => c,
            None => {
                klave::notifier::send_string(&format!("ERROR: network not found"));
                return;
            }
        };
        let eth_address = match v["eth_address"].as_str() {
            Some(c) => c,
            None => {
                klave::notifier::send_string(&format!("ERROR: eth_address not found"));
                return;
            }
        };

        let mut wallet = match Wallet::load(eth_address) {
            Ok(w) => w,
            Err(e) => {
                klave::notifier::send_string(&format!("ERROR: failed to load wallet: {}", e));
                return;
            }
        };

        let nm = match Networks::load() {
            Ok(nm) => nm,
            Err(e) => {
                klave::notifier::send_string(&format!("ERROR: failed to load network manager: {}. Create one first.", e));                
                return
            }
        };

        match wallet.sign_and_send(&nm, network_name, tx, false) {
            Ok(result) => klave::notifier::send_string(&result),
            Err(e) => klave::notifier::send_string(&format!("ERROR: failed to send transaction: {}", e))
        }
    }

    fn wallet_deploy_contract(cmd: String){
        let Ok(v) = serde_json::from_str::<Value>(&cmd) else {
            klave::notifier::send_string(&format!("ERROR: failed to parse '{}' as json", cmd));
            return;
        };    
    
        let chain_id = match v["chainId"].as_u64() {
            Some(c) => c,
            None => {
                klave::notifier::send_string(&format!("ERROR: chainId not found"));
                return;
            }
        };
        let nonce = match v["nonce"].as_u64() {
            Some(n) => n,
            None => {
                klave::notifier::send_string(&format!("ERROR: nonce not found"));
                return;
            }
        };
        let gas_limit = match v["gasLimit"].as_u64(){
            Some(g) => g,
            None => {
                klave::notifier::send_string(&format!("ERROR: gasLimit not found"));
                return;
            }
        };
        let input = match v["input"].as_str() {
            Some(v) => v,            
            None => {
                klave::notifier::send_string(&format!("ERROR: data not found"));
                return;
            }
        };
        let max_fee_per_gas = match v["maxFeePerGas"].as_u64() {
            Some(m) => m,
            None => {
                klave::notifier::send_string(&format!("ERROR: maxFeePerGas not found"));
                return;
            }
        };
        let max_priority_fee_per_gas = match v["maxPriorityFeePerGas"].as_u64() {
            Some(m) => m,
            None => {
                klave::notifier::send_string(&format!("ERROR: maxPriorityFeePerGas not found"));
                return;
            }
        };
    
        let tx = TxEip1559 {
            chain_id: chain_id,
            nonce: nonce,
            gas_limit: gas_limit,
            to: TxKind::Create,
            value: U256::default(),
            input: hex::decode(input.trim_start_matches("0x")).unwrap().into(),
            max_fee_per_gas: max_fee_per_gas as u128,
            max_priority_fee_per_gas: max_priority_fee_per_gas as u128,
            access_list: AccessList::default(),
        };

        let network_name = match v["network_name"].as_str() {
            Some(c) => c,
            None => {
                klave::notifier::send_string(&format!("ERROR: network not found"));
                return;
            }
        };
        let eth_address = match v["eth_address"].as_str() {
            Some(c) => c,
            None => {
                klave::notifier::send_string(&format!("ERROR: eth_address not found"));
                return;
            }
        };

        let mut wallet = match Wallet::load(eth_address) {
            Ok(w) => w,
            Err(e) => {
                klave::notifier::send_string(&format!("ERROR: failed to load wallet: {}", e));
                return;
            }
        };

        let nm = match Networks::load() {
            Ok(nm) => nm,
            Err(e) => {
                klave::notifier::send_string(&format!("ERROR: failed to load network manager: {}. Create one first.", e));                
                return
            }
        };

        let trace = match v["trace"].as_bool() {
            Some(c) => c,
            None => {
                klave::notifier::send_string(&format!("ERROR: trace not found"));
                return;
            }
        };

        match wallet.sign_and_send(&nm, network_name, tx, trace) {
            Ok(result) => klave::notifier::send_string(&result),
            Err(e) => klave::notifier::send_string(&format!("ERROR: failed to send transaction: {}", e))
        }
    }

    fn wallet_balance(cmd: String){
        let Ok(v) = serde_json::from_str::<Value>(&cmd) else {
            klave::notifier::send_string(&format!("ERROR: failed to parse '{}' as json", cmd));
            return;
        };

        let network_name = match v["network_name"].as_str() {
            Some(c) => c,
            None => {
                klave::notifier::send_string(&format!("ERROR: network not found"));
                return;
            }
        };
        let eth_address = match v["eth_address"].as_str() {
            Some(c) => c,
            None => {
                klave::notifier::send_string(&format!("ERROR: eth_address not found"));
                return;
            }
        };

        let wallet = match Wallet::load(eth_address) {
            Ok(w) => w,
            Err(e) => {
                klave::notifier::send_string(&format!("ERROR: failed to load wallet: {}", e));
                return;
            }
        };

        let nm = match Networks::load() {
            Ok(nm) => nm,
            Err(e) => {
                klave::notifier::send_string(&format!("ERROR: failed to load network manager: {}. Create one first.", e));                
                return
            }
        };
        
        match wallet.get_balance(&nm, network_name) {            
            Ok(result) => klave::notifier::send_string(&result),
            Err(e) => klave::notifier::send_string(&format!("ERROR: failed to send balance: {}", e))
        }
    }    

    fn wallet_call_contract(cmd: String) {
        let Ok(v) = serde_json::from_str::<Value>(&cmd) else {
            klave::notifier::send_string(&format!("ERROR: failed to parse '{}' as json", cmd));
            return;
        };

        let contract_owner_address = match v["contract_owner_address"].as_str() {
            Some(c) => match c.parse::<Address>() {
                Ok(a) => a,
                Err(e) => {
                    klave::notifier::send_string(&format!("ERROR: failed to parse contract owner address: {}", e));
                    return;
                }
            },
            None => {
                klave::notifier::send_string(&format!("ERROR: contract owner address not found"));
                return;
            }
        };

        let contract_address = match v["contract_address"].as_str() {
            Some(c) => match c.parse::<Address>() {
                Ok(a) => a,
                Err(e) => {
                    klave::notifier::send_string(&format!("ERROR: failed to parse contract address: {}", e));
                    return;
                }
            },
            None => {
                klave::notifier::send_string(&format!("ERROR: contract address not found"));
                return;
            }
        };

        let recipient_address = match v["recipient_address"].as_str() {
            Some(c) => match c.parse::<Address>() {
                Ok(a) => a,
                Err(e) => {
                    klave::notifier::send_string(&format!("ERROR: failed to parse recipient address: {}", e));
                    return;
                }
            },
            None => {
                klave::notifier::send_string(&format!("ERROR: recipient address not found"));
                return;
            }
        };
        let value = match v["value"].as_str() {
            Some(c) => {
                match U256::from_str_radix(c.trim_start_matches("0x"), 16) {
                    Ok(v) => v,
                    Err(e) => {
                        klave::notifier::send_string(&format!("ERROR: failed to parse value: {}", e));
                        return;
                    }
                }
            },
            None => {
                klave::notifier::send_string(&format!("ERROR: value not found"));
                return;
            }
        };
        let mut hex_encoded_call = String::new();
        match v["input"].as_str() {
            Some(d) => {
                match d {
                    "mint" => {
                            hex_encoded_call = hex::encode(mintCall::new((recipient_address, value)).abi_encode());                            
                        },
                    "burn" => {
                            hex_encoded_call = hex::encode(burnCall::new((recipient_address, value)).abi_encode());                            
                        },
                    _ => {
                        klave::notifier::send_string(&format!("ERROR: unsupported function call"));
                        return;
                    }
                }
            },
            None => {}
        };    
        
        let chain_id = match v["chainId"].as_u64() {
            Some(c) => c,
            None => {
                klave::notifier::send_string(&format!("ERROR: chainId not found"));
                return;
            }
        };
        let nonce = match v["nonce"].as_u64() {
            Some(n) => n,
            None => {
                klave::notifier::send_string(&format!("ERROR: nonce not found"));
                return;
            }
        };
        let gas_limit = match v["gasLimit"].as_u64(){
            Some(g) => g,
            None => {
                klave::notifier::send_string(&format!("ERROR: gasLimit not found"));
                return;
            }
        };
        let max_fee_per_gas = match v["maxFeePerGas"].as_u64() {
            Some(m) => m,
            None => {
                klave::notifier::send_string(&format!("ERROR: maxFeePerGas not found"));
                return;
            }
        };
        let max_priority_fee_per_gas = match v["maxPriorityFeePerGas"].as_u64() {
            Some(m) => m,
            None => {
                klave::notifier::send_string(&format!("ERROR: maxPriorityFeePerGas not found"));
                return;
            }
        };
    
        let tx = TxEip1559 {
            chain_id: chain_id,
            nonce: nonce,
            gas_limit: gas_limit,
            to: TxKind::Call(contract_address),
            value: U256::default(),
            input: hex::decode(hex_encoded_call).unwrap().into(),
            max_fee_per_gas: max_fee_per_gas as u128,
            max_priority_fee_per_gas: max_priority_fee_per_gas as u128,
            ..Default::default()
        };

        let network_name = match v["network_name"].as_str() {
            Some(c) => c,
            None => {
                klave::notifier::send_string(&format!("ERROR: network not found"));
                return;
            }
        };
        let mut wallet = match Wallet::load(&contract_owner_address.to_string()) {
            Ok(w) => w,
            Err(e) => {
                klave::notifier::send_string(&format!("ERROR: failed to load wallet: {}", e));
                return;
            }
        };

        let nm = match Networks::load() {
            Ok(nm) => nm,
            Err(e) => {
                klave::notifier::send_string(&format!("ERROR: failed to load network manager: {}. Create one first.", e));                
                return
            }
        };

        let trace = match v["trace"].as_bool() {
            Some(c) => c,
            None => false
        };

        match wallet.sign_and_send(&nm, network_name, tx.clone(), trace) {
            Ok(result) => {
                klave::notifier::send_string(&format!("{}", result))
            },
            Err(e) => klave::notifier::send_string(&format!("ERROR: failed to send transaction: {}", e))
        }        
    }

    fn wallets_all_for_user(_cmd: String) {
        let sender = match klave::context::get("sender") {
            Ok(s) => s,
            Err(e) => {
                klave::notifier::send_string(&format!("ERROR: {}", e));
                return;
            }
        };

        let user = match User::load(&sender) {
            Ok(u) => u,
            Err(e) => {
                klave::notifier::send_string(&format!("ERROR: failed to load user: {}", e));
                return;
            }
        };

        let mut wallet_strings: Vec<String> = vec![];
        for wallet_address in user.get_wallets() {
            match Wallet::load(&wallet_address) {
                Ok(wallet) => {
                    wallet_strings.push(wallet.to_string().clone());
                },
                Err(e) => {
                    klave::notifier::send_string(&format!("ERROR: failed to get wallet: {}", e));
                }
            }
        }

        klave::notifier::send_string(&format!("{}", serde_json::to_string(&wallet_strings).unwrap()));
    }

    fn wallets_all(_cmd: String) {
        let mut wallet_strings: Vec<String> = vec![];
        for wallet_address in wallets::Wallets::get().get_list_address() {
            match Wallet::load(&wallet_address.address) {
                Ok(wallet) => {
                    wallet_strings.push(wallet.to_string().clone());
                },
                Err(e) => {
                    klave::notifier::send_string(&format!("ERROR: failed to get wallet: {}", e));
                }
            }
        }

        klave::notifier::send_string(&format!("{}", serde_json::to_string(&wallet_strings).unwrap()));
    }

    fn user_add(_cmd: String){
        let sender = match klave::context::get("sender") {
            Ok(s) => s,
            Err(e) => {
                klave::notifier::send_string(&format!("ERROR: {}", e));
                return;
            }
        };

        let mut users = Users::get();
        let mut found = false;
        for u in users.list() {
            if u.id == sender {
                found = true;
                break;
            }
        }
        if found {
            klave::notifier::send_string(&format!("ERROR: user '{}' already exists", sender));
            return;
        }

        let user = User::get(&sender);
        match user.save() {
            Ok(_) => (),
            Err(e) => {
                klave::notifier::send_string(&format!("ERROR: failed to save user: {}", e));
                return;
            }
        }

        match users.add_user(&user.id) {
            Ok(_) => {
                klave::notifier::send_string(&format!("user '{}' added", user.id));
            },
            Err(e) => {
                klave::notifier::send_string(&format!("ERROR: failed to add user '{}': {}", user.id, e));
            }
        }
    }
            
    fn user_get(_cmd: String) {
        let sender = match klave::context::get("sender") {
            Ok(s) => s,
            Err(e) => {
                klave::notifier::send_string(&format!("ERROR: {}", e));
                return;
            }
        };

        let user = match User::load(&sender) {
            Ok(u) => u,
            Err(e) => {
                klave::notifier::send_string(&format!("ERROR: failed to load user: {}", e));
                return;
            }
        };

        klave::notifier::send_string(&format!("{}", user));
    }
    
    fn user_add_wallet(cmd: String) {
        let sender = match klave::context::get("sender") {
            Ok(s) => s,
            Err(e) => {
                klave::notifier::send_string(&format!("ERROR: {}", e));
                return;
            }
        };

        let Ok(v) = serde_json::from_str::<Value>(&cmd) else {
            klave::notifier::send_string(&format!("ERROR: failed to parse '{}' as json", cmd));
            return;
        };

        let eth_address = match v["eth_address"].as_str() {
            Some(c) => c,
            None => {
                klave::notifier::send_string(&format!("ERROR: eth_address not found"));
                return;
            }
        };

        let mut user = match User::load(&sender) {
            Ok(u) => u,
            Err(e) => {
                klave::notifier::send_string(&format!("ERROR: failed to load user: {}", e));
                return;
            }
        };

        match user.add_wallet(eth_address) {
            Ok(_) => klave::notifier::send_string(&format!("wallet {} added to user {}", eth_address, user.id)),
            Err(e) => klave::notifier::send_string(&format!("ERROR: failed to add wallet {} to user {}: {}", eth_address, user.id, e))
        };
    }
        
    fn users_all(_cmd: String) {
        let mut user_strings: Vec<String> = vec![];
        for user_id in Users::get().list {
            match User::load(&user_id) {
                Ok(user) => {
                    user_strings.push(user.to_string().clone());
                },
                Err(e) => {
                    klave::notifier::send_string(&format!("ERROR: failed to get user: {}", e));
                }
            }
        }

        klave::notifier::send_string(&format!("{}", serde_json::to_string(&user_strings).unwrap()));
    }
    
    fn transaction_add(cmd: String) {
        let Ok(v) = serde_json::from_str::<Value>(&cmd) else {
            klave::notifier::send_string(&format!("ERROR: failed to parse '{}' as json", cmd));
            return;
        };

        let source_participant = Participant {
            address: match v["source_address"].as_str() {
                Some(c) => c.to_string(),
                None => {
                    klave::notifier::send_string(&format!("ERROR: source_address not found"));
                    return;
                }
            },
            network_name: match v["source_network_name"].as_str() {
                Some(c) => c.to_string(),
                None => {
                    klave::notifier::send_string(&format!("ERROR: source_network_name not found"));
                    return;
                }
            },
            amount : match v["source_amount"].as_str() {
                Some(c) => {
                    match U256::from_str_radix(c.trim_start_matches("0x"), 16) {
                        Ok(v) => v,
                        Err(e) => {
                            klave::notifier::send_string(&format!("ERROR: failed to parse source_amount: {}", e));
                            return;
                        }
                    }
                },
                None => {
                    klave::notifier::send_string(&format!("ERROR: source_amount not found"));
                    return;
                }
            },
        };
        
        let destination_participant = Participant {
            address: match v["destination_address"].as_str() {
                Some(c) => c.to_string(),
                None => {
                    klave::notifier::send_string(&format!("ERROR: destination_address not found"));
                    return;
                }
            },
            network_name: match v["destination_network_name"].as_str() {
                Some(c) => c.to_string(),
                None => {
                    klave::notifier::send_string(&format!("ERROR: destination_network_name not found"));
                    return;
                }
            },
            amount : match v["destination_amount"].as_str() {
                Some(c) => {
                    match U256::from_str_radix(c.trim_start_matches("0x"), 16) {
                        Ok(v) => v,
                        Err(e) => {
                            klave::notifier::send_string(&format!("ERROR: failed to parse destination_amount: {}", e));
                            return;
                        }
                    }
                },
                None => {
                    klave::notifier::send_string(&format!("ERROR: destination_amount not found"));
                    return;
                }
            },
        };

        let payment_vs_payment = PaymentVsPayment {
            source: source_participant,
            destination: destination_participant,            
            state_machine: PvPstate::Init,
            network_transactions: Vec::<NetworkTransaction>::new()
        };

        let tx = match Transaction::new(&payment_vs_payment) {
            Ok(t) => t,
            Err(e) => {
                klave::notifier::send_string(&format!("ERROR: failed to create transaction: {}", e));
                return;
            }
        };
        // tx.process();
        match tx.save() {
            Ok(_) => (),
            Err(e) => {
                klave::notifier::send_string(&format!("ERROR: failed to save transaction: {}", e));
                return;
            }
        }

        let mut transactions = Transactions::get();
        match transactions.add_transaction(&tx.id) {
            Ok(_) => {
                klave::notifier::send_string(&format!("transaction '{}' added", tx.id));
            },
            Err(e) => {
                klave::notifier::send_string(&format!("ERROR: failed to add transaction '{}': {}", tx.id, e));
            }
        }
    }
    
    fn transaction_get(cmd: String) {
        let Ok(v) = serde_json::from_str::<Value>(&cmd) else {
            klave::notifier::send_string(&format!("ERROR: failed to parse '{}' as json", cmd));
            return;
        };

        let tx_id = match v["tx_id"].as_str() {
            Some(c) => c,
            None => {
                klave::notifier::send_string(&format!("ERROR: tx_id not found"));
                return;
            }
        };

        let tx = match Transaction::load(&tx_id) {
            Ok(t) => t,
            Err(e) => {
                klave::notifier::send_string(&format!("ERROR: failed to load transaction: {}", e));
                return;
            }
        };

        klave::notifier::send_string(&format!("{}", tx));
    }
    
    fn transaction_commit(cmd: String) {
        let Ok(v) = serde_json::from_str::<Value>(&cmd) else {
            klave::notifier::send_string(&format!("ERROR: failed to parse '{}' as json", cmd));
            return;
        };

        let sender = match klave::context::get("sender") {
            Ok(s) => s,
            Err(e) => {
                klave::notifier::send_string(&format!("ERROR: {}", e));
                return;
            }
        };

        let tx_id = match v["tx_id"].as_str() {
            Some(c) => c,
            None => {
                klave::notifier::send_string(&format!("ERROR: tx_id not found"));
                return;
            }
        };

        let participant = match User::load(&sender) {
            Ok(u) => u,
            Err(e) => {
                klave::notifier::send_string(&format!("ERROR: failed to load user: {}", e));
                return;
            }
        };

        //Check if the user is a participant in the transaction
        let mut found = false;
        for tx_role in participant.get_transactions() {
            if tx_role.transaction_id == tx_id {
                found = true;
                break;
            }
        }
        if !found {
            klave::notifier::send_string(&format!("ERROR: user '{}' is not a participant in transaction '{}'", sender, tx_id));
            return;
        }

        let mut tx = match Transaction::load(&tx_id) {
            Ok(t) => t,
            Err(e) => {
                klave::notifier::send_string(&format!("ERROR: failed to load transaction: {}", e));
                return;
            }
        };

        match tx.payment_vs_payment {
            Some(mut pvp) => {
                match pvp.state_machine {
                    PvPstate::Init => {
                        klave::notifier::send_string(&format!("ERROR: transaction is not in the correct state to process payment"));
                    },
                    PvPstate::AwaitingSourceReceive => {
                        //Find the source address in the participant wallets list
                        let mut found = false;
                        for wallet_str in participant.get_wallets() {
                            if wallet_str == pvp.source.address {
                                found = true;
                                break;
                            }
                        }
                        if !found {
                            klave::notifier::send_string(&format!("ERROR: source address '{}' not found in participant wallets", pvp.source.address));
                            return;
                        }
                        let source_address = match v["source_address"].as_str() {
                            Some(c) => c.to_string(),
                            None => {
                                klave::notifier::send_string(&format!("ERROR: source_address not found"));
                                return;
                            }
                        };
                        if source_address != pvp.source.address {
                            klave::notifier::send_string(&format!("ERROR: source_address '{}' does not match transaction source address '{}'", source_address, pvp.source.address));
                            return;
                        }
                        let source_network_name = match v["source_network_name"].as_str() {
                            Some(c) => c.to_string(),
                            None => {
                                klave::notifier::send_string(&format!("ERROR: source_network_name not found"));
                                return;
                            }
                        };
                        if source_network_name != pvp.source.network_name {
                            klave::notifier::send_string(&format!("ERROR: source_network_name '{}' does not match transaction source network name '{}'", source_network_name, pvp.source.network_name));
                            return;
                        };
                        let source_amount = match v["source_amount"].as_str() {
                            Some(c) => {
                                match U256::from_str_radix(c.trim_start_matches("0x"), 16) {
                                    Ok(v) => v,
                                    Err(e) => {
                                        klave::notifier::send_string(&format!("ERROR: failed to parse source_amount: {}", e));
                                        return;
                                    }
                                }
                            },
                            None => {
                                klave::notifier::send_string(&format!("ERROR: source_amount not found"));
                                return;
                            }
                        };
                        if source_amount != pvp.source.amount {
                            klave::notifier::send_string(&format!("ERROR: source_amount '{}' does not match transaction source amount '{}'", source_amount, pvp.source.amount));
                            return;
                        };
                        let escrow_address = match v["escrow_address"].as_str() {
                            Some(c) => c.to_string(),
                            None => {
                                klave::notifier::send_string(&format!("ERROR: escrow_address not found"));
                                return;
                            }
                        };
                        if escrow_address != tx.escrow_address {
                            klave::notifier::send_string(&format!("ERROR: escrow_address '{}' does not match transaction escrow address '{}'", escrow_address, tx.escrow_address));
                            return;
                        }
                        let tx_hash = match v["tx_hash"].as_str() {
                            Some(c) => c.to_string(),
                            None => {
                                klave::notifier::send_string(&format!("ERROR: tx_hash not found"));
                                return;
                            }
                        };

                        pvp.state_machine = PvPstate::AwaitingSourceReceiveFinalized;
                        pvp.network_transactions.push(NetworkTransaction {
                            state: PvPstate::AwaitingSourceReceive,
                            network_name: pvp.source.network_name.clone(),
                            tx_hash: tx_hash.clone()
                        });
                        tx.payment_vs_payment = Some(pvp);                                
                        match tx.save() {
                            Ok(_) => (),
                            Err(e) => {
                                klave::notifier::send_string(&format!("ERROR: failed to save transaction: {}", e));
                                return;
                            }
                        }
                        klave::notifier::send_string(&tx_hash);
                    },
                    PvPstate::AwaitingSourceReceiveFinalized => {
                        klave::notifier::send_string(&format!("ERROR: transaction is not in the correct state to process payment"));
                    },
                    PvPstate::AwaitingDestinationReceive => {
                        //Find the source address in the participant wallets list
                        let mut found = false;
                        for wallet_str in participant.get_wallets() {
                            if wallet_str == pvp.destination.address {
                                found = true;
                                break;
                            }
                        }
                        if !found {
                            klave::notifier::send_string(&format!("ERROR: source address '{}' not found in participant wallets", pvp.destination.address));
                            return;
                        }                        
                        let source_address = match v["source_address"].as_str() {
                            Some(c) => c.to_string(),
                            None => {
                                klave::notifier::send_string(&format!("ERROR: source_address not found"));
                                return;
                            }
                        };
                        if source_address != pvp.destination.address {
                            klave::notifier::send_string(&format!("ERROR: source_address '{}' does not match transaction source address '{}'", source_address, pvp.destination.address));
                            return;
                        }
                        let source_network_name = match v["source_network_name"].as_str() {
                            Some(c) => c.to_string(),
                            None => {
                                klave::notifier::send_string(&format!("ERROR: source_network_name not found"));
                                return;
                            }
                        };
                        if source_network_name != pvp.destination.network_name {
                            klave::notifier::send_string(&format!("ERROR: source_network_name '{}' does not match transaction source network name '{}'", source_network_name, pvp.destination.network_name));
                            return;
                        };
                        let source_amount = match v["source_amount"].as_str() {
                            Some(c) => {
                                match U256::from_str_radix(c.trim_start_matches("0x"), 16) {
                                    Ok(v) => v,
                                    Err(e) => {
                                        klave::notifier::send_string(&format!("ERROR: failed to parse source_amount: {}", e));
                                        return;
                                    }
                                }
                            },
                            None => {
                                klave::notifier::send_string(&format!("ERROR: source_amount not found"));
                                return;
                            }
                        };
                        if source_amount != pvp.destination.amount {
                            klave::notifier::send_string(&format!("ERROR: source_amount '{}' does not match transaction source amount '{}'", source_amount, pvp.destination.amount));
                            return;
                        };            
                        let escrow_address = match v["escrow_address"].as_str() {
                            Some(c) => c.to_string(),
                            None => {
                                klave::notifier::send_string(&format!("ERROR: escrow_address not found"));
                                return;
                            }
                        };
                        if escrow_address != tx.escrow_address {
                            klave::notifier::send_string(&format!("ERROR: escrow_address '{}' does not match transaction escrow address '{}'", escrow_address, tx.escrow_address));
                            return;
                        }
                        match klave::crypto::random::get_random_bytes(size_of::<SecretKey>() as i32) {                        
                            Ok(result) => {
                                pvp.state_machine = PvPstate::AwaitingDestinationReceiveFinalized;
                                pvp.network_transactions.push(NetworkTransaction {
                                    state: PvPstate::AwaitingDestinationReceive,
                                    network_name: pvp.destination.network_name.clone(),
                                    tx_hash: format!("0x{}", hex::encode(result.clone()))
                                });
                                tx.payment_vs_payment = Some(pvp);                                
                                match tx.save() {
                                    Ok(_) => (),
                                    Err(e) => {
                                        klave::notifier::send_string(&format!("ERROR: failed to save transaction: {}", e));
                                        return;
                                    }
                                }
                                klave::notifier::send_string(&hex::encode(result.clone()));
                            },
                            Err(e) => klave::notifier::send_string(&format!("ERROR: failed to send transaction: {}", e))
                        }
                    },
                    PvPstate::AwaitingDestinationReceiveFinalized => {
                        klave::notifier::send_string(&format!("ERROR: transaction is not in the correct state to process payment"));
                    },
                    PvPstate::AwaitingDestinationSend => {
                        //Find the destination address in the participant wallets list
                        let mut found = false;
                        for wallet_str in participant.get_wallets() {
                            if wallet_str == tx.escrow_address {
                                found = true;
                                break;
                            }
                        }
                        if !found {
                            klave::notifier::send_string(&format!("ERROR: escrow address '{}' not found in orchestrator wallets", tx.escrow_address));
                            return;
                        }
                        let wallet = match Wallet::load(&tx.escrow_address) {
                            Ok(w) => w,
                            Err(e) => {
                                klave::notifier::send_string(&format!("ERROR: failed to load wallet: {}", e));
                                return;
                            }
                        };
                        let destination_address = match v["source_address"].as_str() {
                            Some(c) => c.to_string(),
                            None => {
                                klave::notifier::send_string(&format!("ERROR: destination_address not found"));
                                return;
                            }
                        };
                        if destination_address != pvp.destination.address {
                            klave::notifier::send_string(&format!("ERROR: destination_address '{}' does not match transaction destination address '{}'", destination_address, pvp.destination.address));
                            return;
                        }
                        let destination_network_name = match v["source_network_name"].as_str() {
                            Some(c) => c.to_string(),
                            None => {
                                klave::notifier::send_string(&format!("ERROR: destination_network_name not found"));
                                return;
                            }
                        };
                        if destination_network_name != pvp.source.network_name {
                            klave::notifier::send_string(&format!("ERROR: destination_network_name '{}' does not match transaction source network name '{}'", destination_network_name, pvp.source.network_name));
                            return;
                        };
                        let destination_amount = match v["source_amount"].as_str() {
                            Some(c) => {
                                match U256::from_str_radix(c.trim_start_matches("0x"), 16) {
                                    Ok(v) => v,
                                    Err(e) => {
                                        klave::notifier::send_string(&format!("ERROR: failed to parse destination_amount: {}", e));
                                        return;
                                    }
                                }
                            },
                            None => {
                                klave::notifier::send_string(&format!("ERROR: destination_amount not found"));
                                return;
                            }
                        };
                        if destination_amount != pvp.source.amount {
                            klave::notifier::send_string(&format!("ERROR: destination_amount '{}' does not match transaction source amount '{}'", destination_amount, pvp.source.amount));
                            return;
                        };            
                        let escrow_address = match v["escrow_address"].as_str() {
                            Some(c) => c.to_string(),
                            None => {
                                klave::notifier::send_string(&format!("ERROR: escrow_address not found"));
                                return;
                            }
                        };
                        if escrow_address != tx.escrow_address && escrow_address != wallet.get_eth_address() {
                            klave::notifier::send_string(&format!("ERROR: escrow_address '{}' does not match transaction escrow address '{}'", destination_address, pvp.destination.address));
                            return;
                        }

                        let tx_hash = match v["tx_hash"].as_str() {
                            Some(c) => c.to_string(),
                            None => {
                                klave::notifier::send_string(&format!("ERROR: tx_hash not found"));
                                return;
                            }
                        };

                        pvp.state_machine = PvPstate::AwaitingDestinationSendFinalized;
                        pvp.network_transactions.push(NetworkTransaction {
                            state: PvPstate::AwaitingDestinationSend,
                            network_name: pvp.source.network_name.clone(),
                            tx_hash: tx_hash.clone()
                        });
                        tx.payment_vs_payment = Some(pvp);                                
                        match tx.save() {
                            Ok(_) => (),
                            Err(e) => {
                                klave::notifier::send_string(&format!("ERROR: failed to save transaction: {}", e));
                                return;
                            }
                        }
                        klave::notifier::send_string(&tx_hash)
                    },
                    PvPstate::AwaitingDestinationSendFinalized => {
                        klave::notifier::send_string(&format!("ERROR: transaction is not in the correct state to process payment"));
                    },
                    PvPstate::AwaitingSourceSend => {
                        //Find the destination address in the participant wallets list
                        let mut found = false;
                        for wallet_str in participant.get_wallets() {
                            if wallet_str == tx.escrow_address {
                                found = true;
                                break;
                            }
                        }
                        if !found {
                            klave::notifier::send_string(&format!("ERROR: escrow address '{}' not found in orchestrator wallets", tx.escrow_address));
                            return;
                        }
                        let destination_address = match v["source_address"].as_str() {
                            Some(c) => c.to_string(),
                            None => {
                                klave::notifier::send_string(&format!("ERROR: destination_address not found"));
                                return;
                            }
                        };
                        if destination_address != pvp.source.address {
                            klave::notifier::send_string(&format!("ERROR: destination_address '{}' does not match transaction source address '{}'", destination_address, pvp.source.address));
                            return;
                        }
                        let destination_network_name = match v["source_network_name"].as_str() {
                            Some(c) => c.to_string(),
                            None => {
                                klave::notifier::send_string(&format!("ERROR: destination_network_name not found"));
                                return;
                            }
                        };
                        if destination_network_name != pvp.destination.network_name {
                            klave::notifier::send_string(&format!("ERROR: destination_network_name '{}' does not match transaction destination network name '{}'", destination_network_name, pvp.destination.network_name));
                            return;
                        };
                        let destination_amount = match v["source_amount"].as_str() {
                            Some(c) => {
                                match U256::from_str_radix(c.trim_start_matches("0x"), 16) {
                                    Ok(v) => v,
                                    Err(e) => {
                                        klave::notifier::send_string(&format!("ERROR: failed to parse destination_amount: {}", e));
                                        return;
                                    }
                                }
                            },
                            None => {
                                klave::notifier::send_string(&format!("ERROR: destination_amount not found"));
                                return;
                            }
                        };
                        if destination_amount != pvp.destination.amount {
                            klave::notifier::send_string(&format!("ERROR: destination_amount '{}' does not match transaction destination amount '{}'", destination_amount, pvp.destination.amount));
                            return;
                        };            
                        let escrow_address = match v["escrow_address"].as_str() {
                            Some(c) => c.to_string(),
                            None => {
                                klave::notifier::send_string(&format!("ERROR: escrow_address not found"));
                                return;
                            }
                        };
                        if escrow_address != tx.escrow_address {
                            klave::notifier::send_string(&format!("ERROR: escrow_address '{}' does not match transaction escrow address '{}'", escrow_address, tx.escrow_address));
                            return;
                        }
                        match klave::crypto::random::get_random_bytes(size_of::<SecretKey>() as i32) {                        
                            Ok(result) => {
                                pvp.state_machine = PvPstate::AwaitingSourceSendFinalized;
                                pvp.network_transactions.push(NetworkTransaction {
                                    state: PvPstate::AwaitingSourceSend,
                                    network_name: pvp.destination.network_name.clone(),
                                    tx_hash: format!("0x{}", hex::encode(result.clone()))
                                });
                                tx.payment_vs_payment = Some(pvp);                                
                                match tx.save() {
                                    Ok(_) => (),
                                    Err(e) => {
                                        klave::notifier::send_string(&format!("ERROR: failed to save transaction: {}", e));
                                        return;
                                    }
                                }
                                klave::notifier::send_string(&hex::encode(result.clone()));
                            },
                            Err(e) => klave::notifier::send_string(&format!("ERROR: failed to send transaction: {}", e))
                        }
                    },
                    PvPstate::AwaitingSourceSendFinalized => {
                        klave::notifier::send_string(&format!("ERROR: transaction is not in the correct state to process payment"));
                    },
                    PvPstate::Complete => {
                        klave::notifier::send_string(&format!("SUCCESS: transaction is already complete"));
                    },
                    PvPstate::Cancelled => {}
                }
            },
            None => klave::notifier::send_string(&format!("ERROR: transaction does not have a payment"))
        };
    }
    
    fn transaction_apply(cmd: String) {
        let Ok(v) = serde_json::from_str::<Value>(&cmd) else {
            klave::notifier::send_string(&format!("ERROR: failed to parse '{}' as json", cmd));
            return;
        };

        let sender = match klave::context::get("sender") {
            Ok(s) => s,
            Err(e) => {
                klave::notifier::send_string(&format!("ERROR: {}", e));
                return;
            }
        };

        let tx_id = match v["tx_id"].as_str() {
            Some(c) => c,
            None => {
                klave::notifier::send_string(&format!("ERROR: tx_id not found"));
                return;
            }
        };

        let participant = match User::load(&sender) {
            Ok(u) => u,
            Err(e) => {
                klave::notifier::send_string(&format!("ERROR: failed to load user: {}", e));
                return;
            }
        };

        //Check if the user is a participant in the transaction
        let mut found = false;
        for tx_role in participant.get_transactions() {
            if tx_role.transaction_id == tx_id {
                found = true;
                break;
            }
        }
        if !found {
            klave::notifier::send_string(&format!("ERROR: user '{}' is not a participant in transaction '{}'", sender, tx_id));
            return;
        }

        let mut tx = match Transaction::load(&tx_id) {
            Ok(t) => t,
            Err(e) => {
                klave::notifier::send_string(&format!("ERROR: failed to load transaction: {}", e));
                return;
            }
        };

        match tx.payment_vs_payment {
            Some(mut pvp) => {
                match pvp.state_machine {
                    PvPstate::Init => {
                        klave::notifier::send_string(&format!("ERROR: transaction is not in the correct state to process payment"));
                    },
                    PvPstate::AwaitingSourceReceive => {
                        klave::notifier::send_string(&format!("ERROR: transaction is not in the correct state to process payment"));
                    },
                    PvPstate::AwaitingSourceReceiveFinalized => {
                        //check if the tx_hash is in the network_transactions
                        let tx_hash = match v["tx_hash"].as_str() {
                            Some(c) => c.to_string(),
                            None => {
                                klave::notifier::send_string(&format!("ERROR: tx_hash not found"));
                                return;
                            }
                        };
                        let mut found = false;
                        for nt in &mut pvp.network_transactions {
                            if nt.tx_hash == tx_hash {
                                found = true;
                                if nt.state != PvPstate::AwaitingSourceReceive {
                                    klave::notifier::send_string(&format!("ERROR: tx_hash '{}' is not in the correct state to process payment", tx_hash));
                                    return;
                                }
                                nt.state = PvPstate::Complete;
                                break;
                            }
                        }
                        if !found {
                            klave::notifier::send_string(&format!("ERROR: tx_hash '{}' not found in network_transactions", tx_hash));
                            return;
                        }

                        let mut escrow_wallet = match Wallet::load(&tx.escrow_address) {
                            Ok(w) => w,
                            Err(e) => {
                                klave::notifier::send_string(&format!("ERROR: failed to load wallet: {}", e));
                                return;
                            }
                        };
                        match escrow_wallet.mint(&pvp.source.network_name, &pvp.source.amount) {
                            Ok(_) => (),
                            Err(e) => {
                                klave::notifier::send_string(&format!("ERROR: failed to mint: {}", e));
                                return;
                            }
                        }                        
                        let mut source_wallet = match Wallet::load(&pvp.source.address) {
                            Ok(w) => w,
                            Err(e) => {
                                klave::notifier::send_string(&format!("ERROR: failed to load wallet: {}", e));
                                return;
                            }
                        };
                        match source_wallet.burn(&pvp.source.network_name, &pvp.source.amount) {
                            Ok(_) => (),
                            Err(e) => {
                                klave::notifier::send_string(&format!("ERROR: failed to mint: {}", e));
                                return;
                            }
                        }      
                        pvp.state_machine = PvPstate::AwaitingDestinationReceive;
                        tx.payment_vs_payment = Some(pvp.clone());
                        match tx.save() {
                            Ok(_) => {
                                klave::notifier::send_string(&format!("SUCCESS: transaction '{}' finalized", tx.id));
                            },
                            Err(e) => {
                                klave::notifier::send_string(&format!("ERROR: failed to save transaction: {}", e));
                                return;
                            }
                        }                  
                    },
                    PvPstate::AwaitingDestinationReceive => {
                        klave::notifier::send_string(&format!("ERROR: transaction is not in the correct state to process payment"));
                    },
                    PvPstate::AwaitingDestinationReceiveFinalized => {
                        //check if the tx_hash is in the network_transactions
                        let tx_hash = match v["tx_hash"].as_str() {
                            Some(c) => c.to_string(),
                            None => {
                                klave::notifier::send_string(&format!("ERROR: tx_hash not found"));
                                return;
                            }
                        };
                        let mut found = false;
                        for nt in &mut pvp.network_transactions {
                            if nt.tx_hash == tx_hash {
                                found = true;
                                if nt.state != PvPstate::AwaitingDestinationReceive {
                                    klave::notifier::send_string(&format!("ERROR: tx_hash '{}' is not in the correct state to process payment", tx_hash));
                                    return;
                                }
                                nt.state = PvPstate::Complete;
                                break;
                            }
                        }
                        if !found {
                            klave::notifier::send_string(&format!("ERROR: tx_hash '{}' not found in network_transactions", tx_hash));
                            return;
                        }

                        let mut escrow_wallet = match Wallet::load(&tx.escrow_address) {
                            Ok(w) => w,
                            Err(e) => {
                                klave::notifier::send_string(&format!("ERROR: failed to load wallet: {}", e));
                                return;
                            }
                        };
                        match escrow_wallet.mint(&pvp.destination.network_name, &pvp.destination.amount) {
                            Ok(_) => (),
                            Err(e) => {
                                klave::notifier::send_string(&format!("ERROR: failed to mint: {}", e));
                                return;
                            }
                        }                        
                        let mut destination_wallet = match Wallet::load(&pvp.destination.address) {
                            Ok(w) => w,
                            Err(e) => {
                                klave::notifier::send_string(&format!("ERROR: failed to load wallet: {}", e));
                                return;
                            }
                        };
                        match destination_wallet.burn(&pvp.destination.network_name, &pvp.destination.amount) {
                            Ok(_) => (),
                            Err(e) => {
                                klave::notifier::send_string(&format!("ERROR: failed to mint: {}", e));
                                return;
                            }
                        }  
                        pvp.state_machine = PvPstate::AwaitingDestinationSend;
                        tx.payment_vs_payment = Some(pvp);
                        match tx.save() {
                            Ok(_) => {
                                klave::notifier::send_string(&format!("SUCCESS: transaction '{}' finalized", tx.id));
                            },
                            Err(e) => {
                                klave::notifier::send_string(&format!("ERROR: failed to save transaction: {}", e));
                                return;
                            }
                        }                  
                    },
                    PvPstate::AwaitingDestinationSend => {
                        klave::notifier::send_string(&format!("ERROR: transaction is not in the correct state to process payment"));
                    },
                    PvPstate::AwaitingDestinationSendFinalized => {
                        //check if the tx_hash is in the network_transactions
                        let tx_hash = match v["tx_hash"].as_str() {
                            Some(c) => c.to_string(),
                            None => {
                                klave::notifier::send_string(&format!("ERROR: tx_hash not found"));
                                return;
                            }
                        };
                        let mut found = false;
                        for nt in &mut pvp.network_transactions {
                            if nt.tx_hash == tx_hash {
                                found = true;
                                if nt.state != PvPstate::AwaitingDestinationSend {
                                    klave::notifier::send_string(&format!("ERROR: tx_hash '{}' is not in the correct state to process payment", tx_hash));
                                    return;
                                }
                                nt.state = PvPstate::Complete;
                                break;
                            }
                        }
                        if !found {
                            klave::notifier::send_string(&format!("ERROR: tx_hash '{}' not found in network_transactions", tx_hash));
                            return;
                        }

                        let mut escrow_wallet = match Wallet::load(&tx.escrow_address) {
                            Ok(w) => w,
                            Err(e) => {
                                klave::notifier::send_string(&format!("ERROR: failed to load wallet: {}", e));
                                return;
                            }
                        };
                        match escrow_wallet.burn(&pvp.destination.network_name, &pvp.destination.amount) {
                            Ok(_) => (),
                            Err(e) => {
                                klave::notifier::send_string(&format!("ERROR: failed to mint: {}", e));
                                return;
                            }
                        }                        
                        let mut source_wallet = match Wallet::load(&pvp.source.address) {
                            Ok(w) => w,
                            Err(e) => {
                                klave::notifier::send_string(&format!("ERROR: failed to load wallet: {}", e));
                                return;
                            }
                        };
                        match source_wallet.mint(&pvp.destination.network_name, &pvp.destination.amount) {
                            Ok(_) => (),
                            Err(e) => {
                                klave::notifier::send_string(&format!("ERROR: failed to mint: {}", e));
                                return;
                            }
                        }           
                        pvp.state_machine = PvPstate::AwaitingSourceSend;
                        tx.payment_vs_payment = Some(pvp);
                        match tx.save() {
                            Ok(_) => {
                                klave::notifier::send_string(&format!("SUCCESS: transaction '{}' finalized", tx.id));
                            },
                            Err(e) => {
                                klave::notifier::send_string(&format!("ERROR: failed to save transaction: {}", e));
                                return;
                            }
                        }                  
                    },
                    PvPstate::AwaitingSourceSend => {
                        klave::notifier::send_string(&format!("ERROR: transaction is not in the correct state to process payment"));
                    },
                    PvPstate::AwaitingSourceSendFinalized => {
                        //check if the tx_hash is in the network_transactions
                        let tx_hash = match v["tx_hash"].as_str() {
                            Some(c) => c.to_string(),
                            None => {
                                klave::notifier::send_string(&format!("ERROR: tx_hash not found"));
                                return;
                            }
                        };
                        let mut found = false;
                        for nt in &mut pvp.network_transactions {
                            if nt.tx_hash == tx_hash {
                                found = true;
                                if nt.state != PvPstate::AwaitingSourceSend {
                                    klave::notifier::send_string(&format!("ERROR: tx_hash '{}' is not in the correct state to process payment", tx_hash));
                                    return;
                                }
                                nt.state = PvPstate::Complete;
                                break;
                            }
                        }
                        if !found {
                            klave::notifier::send_string(&format!("ERROR: tx_hash '{}' not found in network_transactions", tx_hash));
                            return;
                        }

                        let mut escrow_wallet = match Wallet::load(&tx.escrow_address) {
                            Ok(w) => w,
                            Err(e) => {
                                klave::notifier::send_string(&format!("ERROR: failed to load wallet: {}", e));
                                return;
                            }
                        };
                        match escrow_wallet.burn(&pvp.source.network_name, &pvp.source.amount) {
                            Ok(_) => (),
                            Err(e) => {
                                klave::notifier::send_string(&format!("ERROR: failed to mint: {}", e));
                                return;
                            }
                        }                        
                        let mut destination_wallet = match Wallet::load(&pvp.destination.address) {
                            Ok(w) => w,
                            Err(e) => {
                                klave::notifier::send_string(&format!("ERROR: failed to load wallet: {}", e));
                                return;
                            }
                        };
                        match destination_wallet.mint(&pvp.source.network_name, &pvp.source.amount) {
                            Ok(_) => (),
                            Err(e) => {
                                klave::notifier::send_string(&format!("ERROR: failed to mint: {}", e));
                                return;
                            }                            
                        }    
                        pvp.state_machine = PvPstate::Complete;
                        tx.payment_vs_payment = Some(pvp);
                        match tx.save() {
                            Ok(_) => {
                                klave::notifier::send_string(&format!("SUCCESS: transaction '{}' finalized", tx.id));
                            },
                            Err(e) => {
                                klave::notifier::send_string(&format!("ERROR: failed to save transaction: {}", e));
                                return;
                            }
                        }                  
                    },
                    PvPstate::Complete => {
                        klave::notifier::send_string(&format!("SUCCESS: transaction is already complete"));
                    },
                    PvPstate::Cancelled => {
                        klave::notifier::send_string(&format!("ERROR: transaction is not in the correct state to process payment"));
                    }
                }
            },
            None => klave::notifier::send_string(&format!("ERROR: transaction does not have a payment"))
        };
    }

    fn transactions_all_for_user(_cmd: String) {
        let sender = match klave::context::get("sender") {
            Ok(s) => s,
            Err(e) => {
                klave::notifier::send_string(&format!("ERROR: {}", e));
                return;
            }
        };

        let user = match User::load(&sender) {
            Ok(u) => u,
            Err(e) => {
                klave::notifier::send_string(&format!("ERROR: failed to load user: {}", e));
                return;
            }
        };

        let mut transaction_strings: Vec<String> = vec![];
        for tx_role in user.get_transactions() {
            match Transaction::load(&tx_role.transaction_id) {
                Ok(tx) => {
                    transaction_strings.push(tx.to_string().clone());
                },
                Err(e) => {
                    klave::notifier::send_string(&format!("ERROR: failed to get transaction: {}", e));
                }
            }
        }

        klave::notifier::send_string(&format!("{}", serde_json::to_string(&transaction_strings).unwrap()));
    }

    fn eth_block_number(cmd: String){
        eth::eth_block_number(cmd);
    }

    fn eth_get_block_by_number(cmd: String){
        eth::eth_get_block_by_number(cmd);
    }

    fn eth_gas_price(cmd: String){
        eth::eth_gas_price(cmd);
    }

    fn eth_estimate_gas(cmd: String){
        eth::eth_estimate_gas(cmd);
    }

    fn eth_call_contract(cmd: String){
        eth::eth_call_contract(cmd);
    }

    fn eth_protocol_version(cmd: String){
        eth::eth_protocol_version(cmd);
    }

    fn eth_chain_id(cmd: String){
        eth::eth_chain_id(cmd);
    }

    fn eth_get_transaction_by_hash(cmd: String){
        eth::eth_get_transaction_by_hash(cmd);
    }

    fn eth_get_transaction_receipt(cmd: String){
        eth::eth_get_transaction_receipt(cmd);
    }

    fn eth_get_transaction_count(cmd: String){
        eth::eth_get_transaction_count(cmd);
    }

    fn web_client_version(cmd: String){
        web3::web3_client_version(cmd);
    }

    fn web_sha3(cmd: String){
        web3::web3_sha3(cmd);
    }

    fn net_version(cmd: String){
        web3::net_version(cmd);
    }

    fn get_sender(_cmd: String){
        klave::notifier::send_string(&match klave::context::get("sender") {
            Ok(s) => s,
            Err(e) => e.to_string()
        });
    }

    fn get_trusted_time(_cmd: String){
        klave::notifier::send_string(&match klave::context::get("trusted_time") {
            Ok(s) => s,
            Err(e) => e.to_string()
        });
    }
}

bindings::export!(Component with_types_in bindings);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_solidity_hex_encode() {
        let recipient_address = Address::from_str("0x0E8f8ad443a1270a7D8Af3B30D288DaA0F988e40").unwrap();   
        let value = U256::from_str_radix("0x2386F26FC10000".trim_start_matches("0x"), 16).unwrap();
        let hex_encoded_call = hex::encode(mintCall::new((recipient_address, value)).abi_encode());

        assert_eq!(hex_encoded_call, "40c10f190000000000000000000000000e8f8ad443a1270a7d8af3b30d288daa0f988e40000000000000000000000000000000000000000000000000002386f26fc10000".to_string());
    }
}