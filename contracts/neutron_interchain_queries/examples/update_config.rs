use crate::config::STARGAZE_NFT_ADDRESS;
use cw_orch::daemon::networks::PION_1;
use cw_orch::daemon::DaemonBuilder;

mod config;

use neutron_interchain_queries::NeutronInterchainQueries;

use neutron_interchain_queries::msg::ExecuteMsgFns;

use cw_orch::tokio::runtime::Runtime;

pub fn main() -> cw_orch::anyhow::Result<()> {
    env_logger::init();
    dotenv::dotenv().ok();

    let rt = Runtime::new()?;
    let chain = DaemonBuilder::default()
        .chain(PION_1)
        .handle(rt.handle())
        .build()?;

    let bad_kids = NeutronInterchainQueries::new(chain);

    bad_kids.update_config(Some(STARGAZE_NFT_ADDRESS.to_string()), None)?;

    Ok(())
}
