#![warn(clippy::unwrap_used, clippy::expect_used)]

pub mod contract;
pub mod error;
pub mod ibc;
pub mod mint;
pub mod msg;
pub mod neutron_sdk;
mod query_helpers;
pub mod reply;
pub mod state;
pub mod sudo;

#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod testing;

use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};

pub const CONTRACT_ID: &str = "neutron-bad-ibc";
use cw_orch::interface;
#[interface(InstantiateMsg, ExecuteMsg, QueryMsg, MigrateMsg, id = CONTRACT_ID)]
pub struct NeutronInterchainQueries;

#[cfg(not(target_arch = "wasm32"))]
pub mod interface {
    use cw_orch::prelude::*;

    use crate::NeutronInterchainQueries;

    impl<Chain: CwEnv> Uploadable for NeutronInterchainQueries<Chain> {
        /// Return the path to the wasm file corresponding to the contract
        fn wasm(&self) -> WasmPath {
            artifacts_dir_from_workspace!()
                .find_wasm_path("neutron_interchain_queries")
                .unwrap()
        }
        /// Returns a CosmWasm contract wrapper
        fn wrapper(&self) -> Box<dyn MockContract<Empty>> {
            Box::new(
                ContractWrapper::new_with_empty(
                    crate::contract::execute,
                    crate::contract::instantiate,
                    crate::contract::query,
                )
                .with_migrate(crate::contract::migrate),
            )
        }
    }
}
