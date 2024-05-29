use cosmos_anybuf::{
    interfaces::InterChainQueries,
    neutron::Neutron,
    types::neutron::icq_query::{QueryRegisteredQueryResponse, QueryRegisteredQueryResultResponse},
};
use cosmwasm_std::{Deps, StdError, StdResult};

use prost::Message;

/// Queries raw interchain query result (raw KV storage values or transactions) from Interchain Queries Module.
/// Usually it is better to implement [KVReconstruct] for your own type and then use [query_kv_result],
/// but in cases when Rust forbids to implement foreign trait [KVReconstruct] for some foreign type,
/// it is possible to use [get_raw_interchain_query_result] and reconstruct query result manually.
pub fn get_raw_interchain_query_result(
    deps: Deps,
    interchain_query_id: u64,
) -> StdResult<QueryRegisteredQueryResultResponse> {
    let res = Neutron::query_registered_query_result(&deps.querier, interchain_query_id)?;
    Ok(res)
}

/// Queries registered query info
pub fn get_registered_query(
    deps: Deps,
    interchain_query_id: u64,
) -> StdResult<QueryRegisteredQueryResponse> {
    let res = Neutron::query_registered_query(&deps.querier, interchain_query_id)?;

    Ok(res)
}

/// Decodes protobuf any item into T structure
pub fn decode_message_response<T: Message + Default>(item: &Vec<u8>) -> StdResult<T> {
    let res = T::decode(item.as_slice());
    match res {
        Err(e) => Err(StdError::generic_err(format!("Can't decode item: {}", e))),
        Ok(data) => Ok(data),
    }
}

// Can't import neutron-sdk so integrating used functions here
pub mod sdk {
    use cosmos_sdk_proto::cosmos::base::abci::v1beta1::TxMsgData;
    use cosmwasm_std::{Binary, StdError, StdResult};
    use prost::Message;
    use prost_types::Any;

    const CONTROLLER_PORT_PREFIX: &str = "icacontroller-";
    const ICA_OWNER_DELIMITER: &str = ".";

    /// Constructs a full ICA controller port identifier for a contract with **contract_address** and **interchain_account_id**
    /// <https://github.com/cosmos/ibc-go/blob/46e020640e66f9043c14c53a4d215a5b457d6703/modules/apps/27-interchain-accounts/types/port.go#L11>
    pub fn get_port_id<R: AsRef<str>>(contract_address: R, interchain_account_id: R) -> String {
        CONTROLLER_PORT_PREFIX.to_string()
            + contract_address.as_ref()
            + ICA_OWNER_DELIMITER
            + interchain_account_id.as_ref()
    }
    pub fn decode_acknowledgement_response(data: Binary) -> StdResult<Vec<Any>> {
        let msg_data: Result<TxMsgData, _> = TxMsgData::decode(data.as_slice());
        match msg_data {
            Err(e) => Err(StdError::generic_err(format!(
                "Can't decode response: {}",
                e
            ))),
            Ok(msg) => Ok(msg.msg_responses),
        }
    }
}
