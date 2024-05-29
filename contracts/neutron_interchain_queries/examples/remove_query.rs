use cw_orch::daemon::networks::PION_1;
use cw_orch::daemon::DaemonBuilder;
use cw_orch::tokio::runtime::Runtime;
use neutron_interchain_queries::msg::ExecuteMsgFns;
use neutron_interchain_queries::NeutronInterchainQueries;

pub const STARTGAZE_NFT_ADDRESS: &str = "";

pub const INTERCHAIN_QUERY_ID: &str = "bad-kids:queries";
pub fn main() -> cw_orch::anyhow::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    let rt = Runtime::new()?;
    let chain = DaemonBuilder::default()
        .chain(PION_1)
        .handle(rt.handle())
        .build()?;

    let bad_kids = NeutronInterchainQueries::new(chain);

    const QUERY: u64 = 1;

    // Registering the ica account
    bad_kids.remove_interchain_query(QUERY)?;

    Ok(())
}
