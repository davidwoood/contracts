use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{attr, Addr, BankMsg, Coin, CosmosMsg, SubMsg, Uint128};

use mars_outpost::error::MarsError;
use mars_outpost::incentives::msg::{ExecuteMsg, InstantiateMsg};
use mars_testing::mock_dependencies;

use mars_incentives::contract::{execute, instantiate};
use mars_incentives::state::CONFIG;

use crate::helpers::setup_test;
use mars_incentives::ContractError;

mod helpers;

#[test]
fn test_proper_initialization() {
    let mut deps = mock_dependencies(&[]);

    let info = mock_info("sender", &[]);
    let msg = InstantiateMsg {
        owner: String::from("owner"),
        mars_denom: String::from("umars"),
    };

    let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
    let empty_vec: Vec<SubMsg> = vec![];
    assert_eq!(empty_vec, res.messages);

    let config = CONFIG.load(deps.as_ref().storage).unwrap();
    assert_eq!(config.owner, Addr::unchecked("owner"));
    assert_eq!(config.mars_denom, "umars".to_string());
}

#[test]
fn test_update_config() {
    let mut deps = setup_test();

    // *
    // non owner is not authorized
    // *
    let msg = ExecuteMsg::UpdateConfig {
        owner: None,
        mars_denom: None,
    };
    let info = mock_info("somebody", &[]);
    let error_res = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    assert_eq!(error_res, ContractError::Mars(MarsError::Unauthorized {}));

    // *
    // update config with new params
    // *
    let msg = ExecuteMsg::UpdateConfig {
        owner: Some(String::from("new_owner")),
        mars_denom: None,
    };
    let info = mock_info("owner", &[]);

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());

    // Read config from state
    let new_config = CONFIG.load(deps.as_ref().storage).unwrap();
    assert_eq!(new_config.owner, Addr::unchecked("new_owner"));
    assert_eq!(new_config.mars_denom, "umars".to_string());
}

#[test]
fn test_execute_cosmos_msg() {
    let mut deps = setup_test();

    let bank = BankMsg::Send {
        to_address: "destination".to_string(),
        amount: vec![Coin {
            denom: "uluna".to_string(),
            amount: Uint128::new(123456u128),
        }],
    };
    let cosmos_msg = CosmosMsg::Bank(bank);
    let msg = ExecuteMsg::ExecuteCosmosMsg(cosmos_msg.clone());

    // *
    // non owner is not authorized
    // *
    let info = mock_info("somebody", &[]);
    let error_res = execute(deps.as_mut(), mock_env(), info, msg.clone()).unwrap_err();
    assert_eq!(error_res, ContractError::Mars(MarsError::Unauthorized {}));

    // *
    // can execute Cosmos msg
    // *
    let info = mock_info("owner", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(res.messages, vec![SubMsg::new(cosmos_msg)]);
    assert_eq!(res.attributes, vec![attr("action", "outposts/incentives/execute_cosmos_msg")]);
}
