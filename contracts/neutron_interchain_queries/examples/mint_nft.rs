use cw_orch::daemon::networks::PION_1;
use cw_orch::daemon::DaemonBuilder;

use neutron_interchain_queries::msg::ExecuteMsgFns;
use neutron_interchain_queries::NeutronInterchainQueries;

use cw_orch::tokio::runtime::Runtime;
pub const TOKEN_ID: &str = "80";

mod config;
pub fn main() -> cw_orch::anyhow::Result<()> {
    env_logger::init();
    dotenv::dotenv().ok();

    let rt = Runtime::new()?;
    let chain = DaemonBuilder::default()
        .chain(PION_1)
        .handle(rt.handle())
        .build()?;

    let bad_kids = NeutronInterchainQueries::new(chain);

    // Actually mint the bad kid on the local chain
    bad_kids.mint_nft(TOKEN_ID.to_string())?;

    Ok(())
}
