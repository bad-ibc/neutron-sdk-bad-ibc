use cosmwasm_std::coins;
use cw_orch::daemon::networks::PION_1;
use cw_orch::daemon::DaemonBuilder;

use neutron_interchain_queries::msg::ExecuteMsgFns;
use neutron_interchain_queries::NeutronInterchainQueries;

use cw_orch::tokio::runtime::Runtime;

pub const SENDER: &str = "stars18yj2mc7hjk2zqtwr9exfyj625kffwmjg3dr7tv";
pub const TOKEN_ID: &str = "80";
mod config;

pub fn main() -> cw_orch::anyhow::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    let rt = Runtime::new()?;
    let chain = DaemonBuilder::default()
        .chain(PION_1)
        .handle(rt.handle())
        .build()?;

    let bad_kids = NeutronInterchainQueries::new(chain);

    // Allow for the transfer of the nft from the distant to the local chain
    bad_kids.register_transfer_nft_query(
        0,
        SENDER.to_string(),
        TOKEN_ID.to_string(),
        &coins(1_000_000, "untrn"),
    )?;

    Ok(())
}
