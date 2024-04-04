use crate::error::ContractResult;
use crate::state::{MINTED_TOKENS, TOTAL_MINTED_TOKENS};
use cosmos_anybuf::types::coin::Coin;
use cosmos_anybuf::types::neutron::tokenfactory_tx::{MsgCreateDenom, MsgMint};
use cosmos_anybuf::StargateMsg;
use cosmwasm_std::{Addr, Deps, DepsMut, Env, Response, StdError, StdResult};

const MINT_AMOUNT: u128 = 100;
pub const THRESHOLD_BURN_AMOUNT: u128 = 50;
const NEUTRON_BECH32_PREFIX: &str = "neutron";
const STARS_BECH32_PREFIX: &str = "stars";

/// This function transfer the addr to a local neutron addr
pub fn any_addr_to_neutron(deps: Deps, addr: String) -> ContractResult<Addr> {
    // TODO, test this snippet
    let (_hrp, data, _variant) = bech32::decode(&addr)?;
    let neutron_addr = bech32::encode(NEUTRON_BECH32_PREFIX, data, bech32::Variant::Bech32)?;
    Ok(deps.api.addr_validate(&neutron_addr)?)
}

pub fn any_addr_to_stars(deps: Deps, addr: Addr) -> ContractResult<String> {
    // TODO, test this snippet
    let (_hrp, data, _variant) = bech32::decode(&addr.as_str())?;
    let stars_addr = bech32::encode(STARS_BECH32_PREFIX, data, bech32::Variant::Bech32)?;
    Ok(stars_addr)
}

// In order to create a token denom that is reasonable and doesn't make assumptions on token name length
// We give each new token transfer an increasing id (see https://docs.neutron.org/neutron/modules/3rdparty/osmosis/tokenfactory/overview)
pub fn format_token_sub_denom(_token_id: String, token_count: u64) -> String {
    format!("{}", _token_id)
}

// In order to create a token denom that is reasonable and doesn't make assumptions on token name length
// We give each new token transfer an increasing id (see https://docs.neutron.org/neutron/modules/3rdparty/osmosis/tokenfactory/overview)
pub fn format_token_denom(env: Env, token_id: String, token_count: u64) -> String {
    let sub_denom = format_token_sub_denom(token_id, token_count);

    format!("factory/{}/{}", env.contract.address, sub_denom)
}

pub fn mint_native_receipt(
    deps: DepsMut,
    env: Env,
    token_id: String,
    addr: Addr,
) -> StdResult<Response> {
    // First we see where we are at in terms of numbers of tokens
    let token_count = TOTAL_MINTED_TOKENS.load(deps.storage).unwrap_or(0);
    TOTAL_MINTED_TOKENS.save(deps.storage, &(token_count + 1))?;

    // MINTED_TOKENS should not contain anything at token id location
    MINTED_TOKENS.update(deps.storage, token_id.clone(), |s| match s {
        None => Ok(token_count),
        Some(_) => Err(StdError::generic_err(format!(
            "Token {}, was already migrated",
            token_id
        ))),
    })?;

    let subdenom = format_token_sub_denom(token_id.clone(), token_count);

    let create_denom = MsgCreateDenom {
        sender: env.contract.address.to_string(),
        subdenom,
    }
    .to_msg();
    let mint_msg = MsgMint {
        sender: env.contract.address.to_string(),
        amount: Coin::new(MINT_AMOUNT, format_token_denom(env, token_id, token_count)),
        mint_to_address: addr.to_string(),
    }
    .to_msg();

    Ok(Response::new()
        .add_message(create_denom)
        .add_message(mint_msg))
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::MockApi;
    use cosmwasm_std::testing::MockQuerier;
    use cosmwasm_std::testing::MockStorage;
    use cosmwasm_std::OwnedDeps;
    use neutron_sdk::NeutronResult;
    use std::marker::PhantomData;

    use crate::error::ContractResult;

    use super::any_addr_to_neutron;

    #[test]
    fn right_address_generation() -> ContractResult<()> {
        let deps = OwnedDeps {
            storage: MockStorage::default(),
            api: MockApi::default(),
            querier: MockQuerier::default(),
            custom_query_type: PhantomData,
        };
        let address = "stars1phaxpevm5wecex2jyaqty2a4v02qj7qmruxmf7";
        let neutron_addr = any_addr_to_neutron(deps.as_ref(), address.to_string())?;
        assert_eq!(
            neutron_addr,
            "neutron1phaxpevm5wecex2jyaqty2a4v02qj7qmnlcycg"
        );

        Ok(())
    }
}
