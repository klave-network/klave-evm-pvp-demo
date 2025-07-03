# Klave App Rust Template
Use this template to scaffold a new Klave Rust application to build on-chain, payment-versus-payment (PVP) interactions on EVM-compatible blockchains.

## 📖 About
Klave aims to make it easy to build and deploy WebAssembly application within Trusted Execution Environments (TEEs) and leverage the latest developments in the [WebAssembly component model](https://github.com/WebAssembly/component-model) and [Wasmtime](https://wasmtime.dev/) runtime.
For a more detailed documentation, please read the [Klave docs](https://docs.klave.com/sdk/latest).

This template is a Rust-based framework for creating on-chain PVP (payment-versus-payment) interactions directly on EVM-compatible blockchains. It provides tools and structures to build trustless and transparent financial interactions.


## 📦 Prerequisites
To use and build this template the following tools must be installed:
- The [Rust Toolchain](https://www.rust-lang.org/tools/install) (incl. rust, rustup, cargo)
- cargo-generate : `cargo install cargo-generate`
- cargo-component : `cargo install cargo-component`
- `wasm32-unknown-unknown` target : `rustup target add wasm32-unknown-unknown`

## 🚴 Usage

### 🪼 Deploy on Klave

[Deploy on Klave](https://app.klave.com/login)

### 🛠️ You can build locally before deploying on Klave

[Learn more about `cargo component` here.](https://github.com/bytecodealliance/cargo-component)

```cargo component build --target wasm32-unknown-unknown --release```

this creates a `target` folder with the built wasm files in `target\wasm32-unknown-unknown\release\`

## 🧩 Wasm component

Klave apps are `wasm component`.
In this template, forty-eight methods are implemented, registered and exposed:
You can see these methods exposed in the `wit` [interface](https://github.com/klave-network/klave-evm-pvp-demo/blob/main/apps/klave-evm-pvp-demo/wit/world.wit):
- `export register-routes: func();`
- `export network-add: func(cmd: string);`
- `.......`
- `export get-trusted-time: func(cmd: string);`

1 - The point of entry of the App is the `lib.rs` file and exposes the mandatory guest `wasm component` implementation. First api `db_setup` allows to record in the ledger the database connection settings, second api `execute_table_encryption` allows to encrypt deterministically your PIIs. Third, fourth and fifth apis are encrypted queries.

```Rust
#[allow(warnings)]
mod bindings;

use bindings::Guest;
use klave;
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
    ///
    ///
    ///
    ///
    ///
    ///
    ///
}

bindings::export!(Component with_types_in bindings);
```
Make sure to register each additional Query or Transaction you want to expose via the `register_routes` method.


## 🧑‍🤝‍🧑 Authors

This template is created by [Klave](https://klave.com) and [Secretarium](https://secretarium.com) team members, with contributions from:

- Jeremie Labbe ([@jlabbeklavo](https://github.com/jlabbeKlavo)) - [Klave](https://klave.com) | [Secretarium](https://secretarium.com)

