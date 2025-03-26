#[cfg(not(feature = "library"))]
use crate::error::ContractError;
use crate::execute::{self, get_latest_order_id, init, update_menu_item};
use crate::migrate;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::query::{
    get_all_restaurants, get_escrow, get_menu_items_for_restaurant, get_order_by_id,
    get_order_cost, get_order_status, get_order_status_by_id, get_orders_for_restaurant,
    get_owners, get_rider, get_rider_by_address, get_user_orders, get_user_restaurants,
    query_platform_config,
};

use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
};
use cw2::set_contract_version;

const CONTRACT_NAME: &str = "tastella";
const CONTRACT_VERSION: &str = "0.1.0";

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(init(
        deps,
        info,
        msg.platform_name,
        msg.platform_description,
        msg.owner_address,
        msg.fee_percentage,
        msg.fee_address,
    )
    .map_err(|e| StdError::generic_err(e.to_string()))?)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, env: Env, msg: MigrateMsg) -> Result<Response, ContractError> {
    match msg {
        MigrateMsg::ConvertToMultiOwner {} => migrate::migrate(deps, env, msg),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::RegisterRestaurant {
            name,
            image_uri,
            restaurant_address,
        } => execute::register_restaurant(deps, info, name, image_uri, restaurant_address),

        ExecuteMsg::RegisterRider { name } => execute::register_rider(deps, info, name),

        ExecuteMsg::CreateOrder {
            restaurant_id,
            items,
        } => execute::create_order(deps, env, info, restaurant_id, items),

        ExecuteMsg::AddMenuItem {
            item_id,
            name,
            price,
            image_uri,
        } => execute::add_menu_item(deps, info, item_id, name, price, image_uri),

        ExecuteMsg::RemoveMenuItem { item_id } => execute::remove_menu_item(deps, info, item_id),

        ExecuteMsg::UpdateMenuItem {
            item_id,
            name,
            price,
            available,
            image_uri,
        } => update_menu_item(deps, info, item_id, name, price, available, image_uri),

        ExecuteMsg::ToggleMenuItemAvailability { item_id } => {
            execute::toggle_menu_item_availability(deps, info, item_id)
        }

        ExecuteMsg::AcceptOrder { order_id } => execute::accept_order(deps, info, order_id),

        ExecuteMsg::DepositFunds { order_id } => execute::deposit_funds(deps, info, order_id),

        ExecuteMsg::AssignRider { order_id, rider_id } => {
            execute::assign_rider(deps, order_id, rider_id)
        }

        ExecuteMsg::ConfirmDelivery { order_id } => {
            execute::confirm_delivery(deps, env, info, order_id)
        }

        ExecuteMsg::AddNewOwner { new_owner } => execute::add_new_owner(deps, info, new_owner),

        ExecuteMsg::RemoveOwner { owner } => execute::remove_owner(deps, info, owner),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetPlatformConfig {} => to_json_binary(&query_platform_config(deps)?),

        QueryMsg::GetOwners {} => to_json_binary(&get_owners(deps)?),

        QueryMsg::GetRestaurants {} => to_json_binary(&get_all_restaurants(deps)?),

        QueryMsg::GetAllSuccessfulOrderStatus { is_delivered } => {
            to_json_binary(&get_order_status(deps, is_delivered)?)
        }

        QueryMsg::GetOrderDetails { id } => to_json_binary(&get_order_by_id(deps, id)?),

        QueryMsg::GetEscrow { order_id } => to_json_binary(&get_escrow(deps, order_id)?),

        QueryMsg::GetMenuItems { restaurant_id } => {
            to_json_binary(&get_menu_items_for_restaurant(deps, restaurant_id)?)
        }

        QueryMsg::GetOrdersFromARestaurant { restaurant_id } => {
            to_json_binary(&get_orders_for_restaurant(deps, restaurant_id)?)
        }

        QueryMsg::GetOrderStatusById { order_id } => {
            to_json_binary(&get_order_status_by_id(deps, order_id)?)
        }

        QueryMsg::GetRiderById { rider_id } => to_json_binary(&get_rider(deps, rider_id)?),

        QueryMsg::GetRiderByAddress { riders_address } => {
            let validated_riders_address = deps.api.addr_validate(&riders_address)?;

            to_json_binary(&get_rider_by_address(deps, validated_riders_address)?)
        }

        QueryMsg::GetUserOwnedRestaurants { owner } => {
            let validated_r_o_add = deps.api.addr_validate(&owner)?;

            to_json_binary(&get_user_restaurants(deps, validated_r_o_add)?)
        }

        QueryMsg::GetUserOrders { address } => {
            let validated_address = deps.api.addr_validate(&address)?;
            let response = get_user_orders(deps, validated_address)?;
            to_json_binary(&response)
        }

        QueryMsg::GetLatestOrderId { address } => {
            let validated_address = deps.api.addr_validate(&address)?;
            to_json_binary(&get_latest_order_id(deps, validated_address)?)
        }

        QueryMsg::GetOrderCost {
            restaurant_id,
            items,
        } => to_json_binary(&get_order_cost(deps, restaurant_id, items)?),
    }
}
