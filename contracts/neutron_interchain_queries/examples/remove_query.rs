use cw_orch::daemon::networks::PION_1;
use cw_orch::daemon::DaemonBuilder;
use cw_orch::prelude::CwOrchInstantiate;
use cw_orch::prelude::CwOrchUpload;
use cw_orch::prelude::TxHandler;
use neutron_interchain_queries::contract::NeutronInterchainQueries;
use neutron_interchain_queries::msg::ExecuteMsgFns;
use neutron_interchain_queries::msg::InstantiateMsg;
use tokio::runtime::Runtime;

pub const STARTGAZE_NFT_ADDRESS: &str = "";

pub const INTERCHAIN_QUERY_ID: &str = "bad-kids:queries";
pub fn main() -> cw_orch::anyhow::Result<()> {
    env_logger::init();
    dotenv::dotenv().ok();

    let rt = Runtime::new()?;
    let chain = DaemonBuilder::default()
        .chain(PION_1)
        .handle(rt.handle())
        .build()?;

    let bad_kids = NeutronInterchainQueries::new(INTERCHAIN_QUERY_ID, chain);

    const QUERY: u64 = 6;

    // Registering the ica account
    bad_kids.remove_interchain_query(QUERY)?;

    Ok(())
}
