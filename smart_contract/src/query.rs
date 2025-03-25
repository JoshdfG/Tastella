#[cfg(not(feature = "library"))]
use crate::state::{Order, OrderStatus, Restaurant, ESCROWS, ORDERS, RESTAURANTS};
use crate::{
    msg::{
        GetEscrowResponse, GetMenuItemsResponse, GetOrderResponse, GetOrdersResponse,
        GetRestaurantsResponse, GetRiderResponse, GetUserOrdersResponse,
        GetUserRestaurantsResponse, PlatformConfigResponse,
    },
    state::{MenuItem, Rider, MENU_ITEMS, PLATFORM_CONFIG, RIDERS},
};

use cosmwasm_std::{Deps, StdError, StdResult};

use crate::msg::GetOrderStatusResponse;

pub fn query_platform_config(deps: Deps) -> StdResult<PlatformConfigResponse> {
    let config = PLATFORM_CONFIG.load(deps.storage)?;
    Ok(PlatformConfigResponse {
        platform_name: config.platform_name,
        platform_description: config.platform_description,
        owner_address: config.owner_address,
        fee_percentage: config.fee_percentage,
        fee_address: config.fee_address,
    })
}

pub fn get_all_restaurants(deps: Deps) -> StdResult<GetRestaurantsResponse> {
    let restaurants: Vec<Restaurant> = RESTAURANTS
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .map(|item| {
            let (_, restaurant) = item?;

            Ok(restaurant)
        })
        .collect::<StdResult<Vec<_>>>()?;

    Ok(GetRestaurantsResponse { restaurants })
}

pub fn get_menu_items_for_restaurant(
    deps: Deps,
    restaurant_id: String,
) -> StdResult<GetMenuItemsResponse> {
    let menu_items: Vec<MenuItem> = MENU_ITEMS
        .prefix(&restaurant_id)
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .map(|item| {
            let (_, menu_item) = item?;
            Ok(menu_item)
        })
        .collect::<StdResult<Vec<_>>>()?;

    Ok(GetMenuItemsResponse { menu_items })
}

pub fn get_orders_for_restaurant(
    deps: Deps,
    restaurant_id: String,
) -> StdResult<GetOrdersResponse> {
    let orders: Vec<Order> = ORDERS
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .map(|item| {
            let (_, order) = item?;
            Ok(order)
        })
        .filter(|order| {
            if let Ok(order) = order {
                order.restaurant_id == restaurant_id
            } else {
                false
            }
        })
        .collect::<StdResult<Vec<_>>>()?;

    Ok(GetOrdersResponse { orders })
}

pub fn get_order_status(deps: Deps, is_delivered: bool) -> StdResult<Vec<Order>> {
    let orders: Vec<Order> = ORDERS
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .map(|item| {
            let (_, order) = item?;

            Ok(order)
        })
        .collect::<StdResult<Vec<_>>>()?;

    let filtered_orders = orders
        .into_iter()
        .filter(|order| match order.status {
            OrderStatus::Completed => is_delivered,
            _ => !is_delivered,
        })
        .collect();

    Ok(filtered_orders)
}

pub fn get_order_status_by_id(deps: Deps, order_id: String) -> StdResult<GetOrderStatusResponse> {
    let order = ORDERS.load(deps.storage, &order_id)?;
    Ok(GetOrderStatusResponse {
        order_id,
        status: order.status,
    })
}

pub fn get_single_order(deps: Deps, order_id: String) -> StdResult<GetOrderResponse> {
    let order = ORDERS.load(deps.storage, &order_id)?;

    Ok(GetOrderResponse { order })
}

pub fn get_order_by_id(deps: Deps, order_id: String) -> StdResult<GetOrderResponse> {
    let order = ORDERS.load(deps.storage, &order_id)?;
    Ok(GetOrderResponse { order })
}

pub fn get_escrow(deps: Deps, order_id: String) -> StdResult<GetEscrowResponse> {
    let escrow = ESCROWS.load(deps.storage, &order_id)?;
    Ok(GetEscrowResponse { escrow })
}

pub fn get_rider(deps: Deps, rider_id: String) -> StdResult<Rider> {
    RIDERS.load(deps.storage, &rider_id)
}

pub fn get_user_restaurants(deps: Deps, owner: String) -> StdResult<GetUserRestaurantsResponse> {
    let owner_addr = deps.api.addr_validate(&owner)?;
    let restaurants: Vec<Restaurant> = RESTAURANTS
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .filter_map(|item| {
            let (_, restaurant) = item.unwrap();
            if restaurant.owner == owner_addr {
                Some(restaurant)
            } else {
                None
            }
        })
        .collect();
    Ok(GetUserRestaurantsResponse { restaurants })
}

pub fn get_rider_by_address(deps: Deps, address: String) -> StdResult<GetRiderResponse> {
    let sender = deps.api.addr_validate(&address)?;
    let rider_id = format!("rider_{}", sender);
    let rider = RIDERS.may_load(deps.storage, &rider_id)?;
    Ok(GetRiderResponse { rider })
}

pub fn get_user_orders(deps: Deps, address: String) -> StdResult<GetUserOrdersResponse> {
    let sender = deps
        .api
        .addr_validate(&address)
        .map_err(|e| StdError::generic_err(format!("Invalid address: {}", e)))?;
    let orders: Vec<Order> = ORDERS
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .filter_map(|item| {
            let (_, order) = item.unwrap();

            if order.customer == sender {
                Some(order)
            } else {
                None
            }
        })
        .collect();
    Ok(GetUserOrdersResponse { orders })
}
