#![allow(dead_code)]

use std::marker::PhantomData;

use cosmwasm_std::testing::{mock_env, MockApi, MockQuerier, MockStorage};
use cosmwasm_std::{coin, from_binary, Coin, Deps, DepsMut, OwnedDeps};
use osmo_bindings::{OsmosisQuery, PoolStateResponse};
use osmosis_std::types::osmosis::gamm::v1beta1::{Pool, PoolAsset, QueryPoolResponse};

use mars_outpost::oracle::{InstantiateMsg, QueryMsg};
use mars_testing::{mock_info, MarsMockQuerier};

use mars_oracle_osmosis::contract::entry;
use mars_oracle_osmosis::msg::ExecuteMsg;
use mars_oracle_osmosis::OsmosisPriceSource;

use osmosis_std::shim::Any;
use prost::Message;

pub fn setup_test() -> OwnedDeps<MockStorage, MockApi, MarsMockQuerier, OsmosisQuery> {
    let mut deps = OwnedDeps::<_, _, _, OsmosisQuery> {
        storage: MockStorage::default(),
        api: MockApi::default(),
        querier: MarsMockQuerier::new(MockQuerier::new(&[])),
        custom_query_type: PhantomData,
    };

    // set a few osmosis pools
    let assets = vec![coin(42069, "uatom"), coin(69420, "uosmo")];
    deps.querier
        .set_query_pool_response(1, prepare_query_pool_response(1, &assets, &[5000u64, 5000u64]));
    deps.querier.set_pool_state_response(
        1,
        PoolStateResponse {
            assets,
            shares: coin(10000, "gamm/pool/1"),
        },
    );
    let assets = vec![coin(12345, "uusdc"), coin(23456, "uatom")];
    deps.querier
        .set_query_pool_response(64, prepare_query_pool_response(64, &assets, &[5000u64, 5000u64]));
    deps.querier.set_pool_state_response(
        64,
        PoolStateResponse {
            assets,
            shares: coin(10000, "gamm/pool/64"),
        },
    );
    let assets = vec![coin(12345, "uosmo"), coin(88888, "umars")];
    deps.querier
        .set_query_pool_response(89, prepare_query_pool_response(89, &assets, &[5000u64, 5000u64]));
    deps.querier.set_pool_state_response(
        89,
        PoolStateResponse {
            assets,
            shares: coin(10000, "gamm/pool/89"),
        },
    );
    let assets = vec![coin(100000, "uusdc"), coin(100000, "uusdt"), coin(100000, "udai")];
    deps.querier.set_query_pool_response(
        3333,
        prepare_query_pool_response(3333, &assets, &[5000u64, 5000u64, 5000u64]),
    );
    deps.querier.set_pool_state_response(
        3333,
        PoolStateResponse {
            assets,
            shares: coin(10000, "gamm/pool/3333"),
        },
    );
    // Set not XYK pool (different assets weights)
    let assets = vec![coin(100000, "uion"), coin(100000, "uosmo")];
    deps.querier.set_query_pool_response(
        4444,
        prepare_query_pool_response(4444, &assets, &[5000u64, 5005u64]),
    );
    deps.querier.set_pool_state_response(
        4444,
        PoolStateResponse {
            assets,
            shares: coin(10000, "gamm/pool/4444"),
        },
    );

    // instantiate the oracle contract
    entry::instantiate(
        deps.as_mut(),
        mock_env(),
        mock_info("owner"),
        InstantiateMsg {
            owner: "owner".to_string(),
            base_denom: "uosmo".to_string(),
        },
    )
    .unwrap();

    deps
}

fn prepare_query_pool_response(
    pool_id: u64,
    assets: &[Coin],
    weights: &[u64],
) -> QueryPoolResponse {
    let pool = Pool {
        address: "address".to_string(),
        id: pool_id,
        pool_params: None,
        future_pool_governor: "future_pool_governor".to_string(),
        total_shares: None,
        pool_assets: prepare_pool_assets(assets, weights),
        total_weight: "".to_string(),
    };

    let mut buf = vec![];
    pool.encode(&mut buf).unwrap();

    QueryPoolResponse {
        pool: Some(Any {
            type_url: "osmosis_std::types::osmosis::gamm::v1beta1::Pool".to_string(),
            value: buf,
        }),
    }
}

fn prepare_pool_assets(assets: &[Coin], weights: &[u64]) -> Vec<PoolAsset> {
    assert_eq!(assets.len(), weights.len());

    assets
        .iter()
        .zip(weights)
        .map(|zipped| {
            let (coin, weight) = zipped;
            PoolAsset {
                token: Some(osmosis_std::types::cosmos::base::v1beta1::Coin {
                    denom: coin.denom.clone(),
                    amount: coin.amount.to_string(),
                }),
                weight: weight.to_string(),
            }
        })
        .collect()
}

pub fn set_price_source(
    deps: DepsMut<OsmosisQuery>,
    denom: &str,
    price_source: OsmosisPriceSource,
) {
    entry::execute(
        deps,
        mock_env(),
        mock_info("owner"),
        ExecuteMsg::SetPriceSource {
            denom: denom.to_string(),
            price_source,
        },
    )
    .unwrap();
}

pub fn query<T: serde::de::DeserializeOwned>(deps: Deps<OsmosisQuery>, msg: QueryMsg) -> T {
    from_binary(&entry::query(deps, mock_env(), msg).unwrap()).unwrap()
}
