#[cfg(not(feature = "library"))]
use crate::error::ContractError;
use crate::msg::MigrateMsg;

use crate::state::{OldPlatformConfig, OldRider, PlatformConfig, Rider, PLATFORM_CONFIG, RIDERS};

use cosmwasm_std::{DepsMut, Env, Response, StdResult};
use cw_storage_plus::{Item, Map};

pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    const OLD_CONFIG: Item<OldPlatformConfig> = Item::new("platform_config");
    let old_config = OLD_CONFIG.load(deps.storage)?;
    let new_config = PlatformConfig {
        platform_name: old_config.platform_name,
        platform_description: old_config.platform_description,
        owners: vec![old_config.owner_address],
        fee_percentage: old_config.fee_percentage,
        fee_address: old_config.fee_address,
    };
    PLATFORM_CONFIG.save(deps.storage, &new_config)?;
    Ok(Response::new()
        .add_attribute("action", "migrate")
        .add_attribute("result", "converted_to_multi_owner"))
}

pub fn update_rider_and_add_user(deps: DepsMut, _env: Env) -> Result<Response, ContractError> {
    const OLD_RIDERS: Map<String, OldRider> = Map::new("riders");
    let old_riders = OLD_RIDERS
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .collect::<StdResult<Vec<_>>>()?;
    for (id, old_rider) in old_riders {
        let new_rider = Rider {
            id: old_rider.id.clone(),
            name: old_rider.name,
            wallet: old_rider.wallet,
            phone_number: String::new(),
            is_registered: old_rider.is_registered,
        };
        RIDERS.save(deps.storage, &id, &new_rider)?;
    }

    Ok(Response::new()
        .add_attribute("action", "update_rider_and_add_user")
        .add_attribute("result", "updated_riders_and_users"))
}
