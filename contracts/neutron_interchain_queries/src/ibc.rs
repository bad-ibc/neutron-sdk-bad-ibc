use crate::error::ContractResult;
use crate::neutron_sdk::sdk::get_port_id;
use crate::{
    reply::SUDO_PAYLOAD_REPLY_ID,
    state::{save_reply_payload, SudoPayload, INTERCHAIN_ACCOUNTS},
};
use cosmos_anybuf::{
    interfaces::InterchainTxs,
    neutron::{IbcFee, Neutron},
};
use cosmwasm_std::{Coin, CosmosMsg, DepsMut, Env, Response, StdResult, SubMsg};

const FEE_DENOM: &str = "untrn";

pub fn execute_register_ica(
    deps: DepsMut,
    env: Env,
    funds: Vec<Coin>,
    connection_id: String,
    interchain_account_id: String,
) -> ContractResult<Response> {
    let register = Neutron::register_interchain_account(
        &env.contract.address,
        connection_id,
        &interchain_account_id,
        funds,
    );
    let key = get_port_id(env.contract.address.as_str(), &interchain_account_id);
    // we are saving empty data here because we handle response of registering ICA in sudo_open_ack method
    INTERCHAIN_ACCOUNTS.save(deps.storage, key, &None)?;
    Ok(Response::new().add_message(register))
}

pub fn min_ntrn_ibc_fee(fee: IbcFee) -> IbcFee {
    IbcFee {
        recv_fee: fee.recv_fee,
        ack_fee: fee
            .ack_fee
            .into_iter()
            .filter(|a| a.denom == FEE_DENOM)
            .collect(),
        timeout_fee: fee
            .timeout_fee
            .into_iter()
            .filter(|a| a.denom == FEE_DENOM)
            .collect(),
    }
}

// saves payload to process later to the storage and returns a SubmitTX Cosmos SubMsg with necessary reply id
pub fn msg_with_sudo_callback<C: Into<CosmosMsg<T>>, T>(
    deps: DepsMut,
    msg: C,
    payload: SudoPayload,
) -> StdResult<SubMsg<T>> {
    save_reply_payload(deps.storage, payload)?;
    Ok(SubMsg::reply_on_success(msg, SUDO_PAYLOAD_REPLY_ID))
}
