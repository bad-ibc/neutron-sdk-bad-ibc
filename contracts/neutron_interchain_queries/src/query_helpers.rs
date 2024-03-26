use cosmwasm_std::{Addr, CosmosMsg, Deps, Env, StdError};
use neutron_sdk::{
    bindings::msg::NeutronMsg,
    interchain_queries::{
        types::{
            QueryPayload, QueryType, TransactionFilterItem, TransactionFilterOp,
            TransactionFilterValue,
        },
        v047::types::WASM_CONTRACT_STORE_PREFIX,
    },
    NeutronError, NeutronResult,
};
use serde_json_wasm::to_string;

use crate::{
    contract::INTERCHAIN_ACCOUNT_ID,
    mint::any_addr_to_stars,
    state::{get_ica, SENDER_TXS, TOKEN_INFOS},
};
use cosmos_anybuf::{
    types::neutron::{icq_tx::MsgRegisterInterchainQuery, interchainqueries::KVKey},
    StargateMsg,
};

// [{"field": "{eventType}.{attributeKey}", "val": "{attributeValue}", "op": "gte"}, ...]
pub const HEIGHT_FIELD: &str = "tx.height";
pub const WASM_EXECUTE_MSG_TYPE: &str = "/cosmwasm.wasm.v1.MsgExecuteContract";

/// Creates a message to register an Interchain Query to get transfer events to a recipient on a remote chain.
///
/// * **connection_id** is an IBC connection identifier between Neutron and remote chain;
/// * **recipient** is an address of an account on remote chain for which you want to get list of transfer transactions;
/// * **update_period** is used to say how often the query must be updated.
/// * **min_height** is used to set min height for query (by default = 0).
pub fn new_register_transfer_nft_query_msg(
    connection_id: String,
    update_period: u64,
    min_height: u64,
    recipient: String,
    sender: String,
    contract_address: String,
    token_id: String,
) -> NeutronResult<CosmosMsg> {
    let query_data = nft_transfer_filter(min_height, recipient, sender, contract_address, token_id);

    // [{"field": "{eventType}.{attributeKey}", "val": "{attributeValue}", "op": "gte"}, ...]

    Ok(MsgRegisterInterchainQuery {
        query_type: QueryType::TX.into(),
        keys: vec![],
        transactions_filter: to_string(&query_data)
            .map_err(|e| StdError::generic_err(e.to_string()))?,
        connection_id,
        update_period,
        sender,
    }
    .to_msg())
}

pub fn nft_transfer_filter(
    min_height: u64,
    recipient: String,
    sender: String,
    contract_address: String,
    token_id: String,
) -> Vec<TransactionFilterItem> {
    let query_data = vec![
        TransactionFilterItem {
            field: HEIGHT_FIELD.to_string(),
            op: TransactionFilterOp::Gte,
            value: TransactionFilterValue::Int(min_height),
        },
        TransactionFilterItem {
            field: "wasm.recipient".to_string(),
            op: TransactionFilterOp::Eq,
            value: TransactionFilterValue::String(recipient),
        },
        TransactionFilterItem {
            field: "wasm.action".to_string(),
            op: TransactionFilterOp::Eq,
            value: TransactionFilterValue::String("transfer_nft".to_string()),
        },
        TransactionFilterItem {
            field: "wasm.sender".to_string(),
            op: TransactionFilterOp::Eq,
            value: TransactionFilterValue::String(sender),
        },
        TransactionFilterItem {
            field: "wasm._contract_address".to_string(),
            op: TransactionFilterOp::Eq,
            value: TransactionFilterValue::String(contract_address),
        },
        TransactionFilterItem {
            field: "wasm.token_id".to_string(),
            op: TransactionFilterOp::Eq,
            value: TransactionFilterValue::String(token_id),
        },
    ];
    query_data
}

pub fn verify_query(
    deps: Deps,
    env: &Env,
    token_id: String,
    requester: Addr,
) -> NeutronResult<String> {
    // verify whether the token has been sent to the ica by the owner
    // verify whether the token is still owned by the ica
    // check_host_state(deps, token_id.clone(), env)?;
    let host_address = check_host_transactions(deps, requester, &token_id)?;

    Ok(host_address)
}

fn check_host_state(deps: Deps, token_id: String, env: &Env) -> Result<(), NeutronError> {
    let token_info = TOKEN_INFOS
        .load(deps.storage, token_id)
        .map_err(|_| NeutronError::Std(StdError::generic_err("Token does not exist")))?;

    let (_, ic_account_addr) = get_ica(deps, env, INTERCHAIN_ACCOUNT_ID)
        .map_err(|_| NeutronError::Std(StdError::generic_err("ICA does not exist")))?;

    if ic_account_addr != token_info.owner.to_string() {
        panic!(
            "Token is not owned by the ICA. is: {}, should be: {}",
            token_info.owner, ic_account_addr
        );
    } else {
        Ok(())
    }
}

fn check_host_transactions(
    deps: Deps,
    requester: Addr,
    token_id: &String,
) -> Result<String, NeutronError> {
    let host_address = any_addr_to_stars(deps, requester)?;
    let _sender_tx = SENDER_TXS
        .load(deps.storage, &host_address)
        .map_err(|_| {
            NeutronError::Std(StdError::generic_err(format!(
                "No key for sender {}",
                host_address
            )))
        })?
        .into_iter()
        .find(|tx| tx.token_id == *token_id);

    if let Some(sender_tx) = _sender_tx {
        if sender_tx.sender != host_address {
            return Err(NeutronError::Std(StdError::generic_err(
                "Sender does not match",
            )));
        }
    } else {
        return Err(NeutronError::Std(StdError::generic_err(
            "No matching transaction found",
        )));
    }

    Ok(host_address)
}

pub fn nft_owned_filter(token_id: String, contract_address: String) -> (Vec<u8>, Vec<u8>) {
    let key = [
        b"\x00",
        &["tokens".len() as u8],
        "tokens".as_bytes(),
        &token_id.as_bytes(),
    ];
    let mut veckey = vec![];
    for k in key.into_iter() {
        veckey.extend_from_slice(k);
    }

    let mut full_store_key: Vec<u8> = vec![WASM_CONTRACT_STORE_PREFIX];
    full_store_key.extend_from_slice(contract_address.as_bytes().as_ref());
    full_store_key.extend_from_slice(&veckey.clone().as_slice());

    // formatted string of the key
    // let token_id_key = format!("tokens{}", token_id);

    return (veckey, full_store_key);
}

#[cfg(test)]
mod tests {
    use crate::{
        state::{NftTransfer, INTERCHAIN_ACCOUNTS},
        testing::mock_querier::mock_dependencies,
    };

    use super::*;
    use cosmwasm_std::{testing::mock_env, DepsMut, Empty};
    use cw721_base::state::TokenInfo;
    use neutron_sdk::interchain_txs::helpers::get_port_id;

    const STARS_ADDR: &str = "stars1nvh5r0hq0jr83f2ka8wdzfga9jazjzcczxe782";
    const NTRN_ADDR: &str = "neutron1nvh5r0hq0jr83f2ka8wdzfga9jazjzccj98pku";
    const INTERCHAIN_ACCOUNT_ADDR: &str = "stars0000000000000000000000000000";
    const COLL_ADDRESS: &str = "stars0000000000000000000000000001";
    const TOKEN_ID_0: &str = "0000";
    const TOKEN_ID_1: &str = "0001";

    fn create_token_info(owner: &str) -> TokenInfo<Empty> {
        TokenInfo {
            owner: Addr::unchecked(owner.to_string()),
            approvals: vec![],
            token_uri: None,
            extension: Empty {},
        }
    }

    fn set_ica_account(deps: DepsMut, contract_address: String, ica_address: &str) {
        let k = get_port_id(contract_address, INTERCHAIN_ACCOUNT_ID.to_string());
        let ica = Some(("".to_string(), ica_address.to_string()));
        INTERCHAIN_ACCOUNTS.save(deps.storage, k, &ica).unwrap();
    }

    #[test]
    fn test_verify_query_success() {
        let mut deps = mock_dependencies(&[]);
        let env = mock_env();
        set_ica_account(
            deps.as_mut(),
            env.contract.address.to_string(),
            INTERCHAIN_ACCOUNT_ADDR,
        );

        // test ica setup
        let _ica = get_ica(deps.as_ref(), &env, INTERCHAIN_ACCOUNT_ID).unwrap();

        let token_info = create_token_info(INTERCHAIN_ACCOUNT_ADDR);

        let nft_transfers: Vec<NftTransfer> = vec![NftTransfer {
            contract_address: COLL_ADDRESS.to_string(),
            token_id: TOKEN_ID_0.to_string(),
            sender: STARS_ADDR.to_string(),
        }];
        // Preset storage
        TOKEN_INFOS
            .save(&mut deps.storage, TOKEN_ID_0.to_string(), &token_info)
            .unwrap();
        SENDER_TXS
            .save(&mut deps.storage, &STARS_ADDR, &nft_transfers)
            .unwrap();

        let result = verify_query(
            deps.as_ref(),
            &env,
            TOKEN_ID_0.to_string(),
            Addr::unchecked(NTRN_ADDR.to_string()),
        );
        assert!(result.is_ok(), "Unexpected error: {:?}", result);

        assert_eq!(
            result.as_ref().unwrap().as_str(),
            STARS_ADDR,
            "result: {:?}, wanted{:?}",
            result.as_ref().unwrap(),
            STARS_ADDR
        );
    }

    #[test]
    fn test_verify_query_failure_invalid_token() {
        let mut deps = mock_dependencies(&[]);
        let env = mock_env();

        // Set up required data
        let token_info = create_token_info(INTERCHAIN_ACCOUNT_ADDR);

        // Preset storage with valid token
        TOKEN_INFOS
            .save(&mut deps.storage, TOKEN_ID_0.to_string(), &token_info)
            .unwrap();

        let result = verify_query(
            deps.as_ref(),
            &env,
            TOKEN_ID_1.to_string(),
            Addr::unchecked(NTRN_ADDR.to_string()),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_query_failure_invalid_owner() {
        let mut deps = mock_dependencies(&[]);
        let env = mock_env();

        // Set up required data
        let invalid_token_info = create_token_info(STARS_ADDR); // INVALID owner here

        // Preset storage
        TOKEN_INFOS
            .save(
                &mut deps.storage,
                TOKEN_ID_0.to_string(),
                &invalid_token_info,
            )
            .unwrap();

        let result = verify_query(
            deps.as_ref(),
            &env,
            TOKEN_ID_0.to_string(),
            Addr::unchecked(NTRN_ADDR.to_string()),
        );
        assert!(result.is_err());
    }
}
