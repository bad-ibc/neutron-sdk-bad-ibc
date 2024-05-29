use crate::config::IBC_CONNECTION_ID;
use crate::config::STARGAZE_NFT_ADDRESS;
use cosmwasm_std::coins;
use cw_orch::daemon::networks::PION_1;
use cw_orch::daemon::DaemonBuilder;
use cw_orch::prelude::ContractInstance;
use cw_orch::prelude::CwOrchInstantiate;
use cw_orch::prelude::CwOrchUpload;
use cw_orch::prelude::TxHandler;
use neutron_interchain_queries::contract::INTERCHAIN_ACCOUNT_ID;
use neutron_interchain_queries::msg::ExecuteMsgFns;
use neutron_interchain_queries::msg::InstantiateMsg;
use neutron_interchain_queries::NeutronInterchainQueries;

use cw_orch::tokio::runtime::Runtime;
mod config;

pub fn main() -> cw_orch::anyhow::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    let rt = Runtime::new()?;
    let chain = DaemonBuilder::default()
        .chain(PION_1)
        .handle(rt.handle())
        .build()?;

    let bad_kids = NeutronInterchainQueries::new(chain.clone());

    println!("You might need this command to create a connection and client : ./hermes --config config.toml create connection --a-chain pion-1 --b-chain elgafar-1",);
    // Uploading
    bad_kids.upload()?;

    // Instantiating the contract
    bad_kids.instantiate(
        &InstantiateMsg {
            connection_id: IBC_CONNECTION_ID.to_string(),
            contract_addr: STARGAZE_NFT_ADDRESS.to_string(),
        },
        Some(&chain.sender()),
        None,
    )?;

    // Registering the ica account
    bad_kids.register_ica(&coins(100_000, "untrn"))?;

    println!("./hermes --config config.toml create channel --a-chain pion-1 --a-connection {} --a-port icacontroller-{}.{} --b-port icahost --order ordered",IBC_CONNECTION_ID, bad_kids.address()?, INTERCHAIN_ACCOUNT_ID );

    Ok(())
}
