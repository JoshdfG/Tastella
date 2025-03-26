#[cfg(not(feature = "library"))]
use crate::error::ContractError;
use crate::msg::MigrateMsg;

use crate::state::{OldPlatformConfig, PlatformConfig, PLATFORM_CONFIG};

use cosmwasm_std::{DepsMut, Env, Response};
use cw_storage_plus::Item;

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
