#[cfg(not(feature = "library"))]
use crate::state::{Order, OrderStatus, Restaurant, ESCROWS, ORDERS, RESTAURANTS};
use crate::{
    msg::{
        GetEscrowResponse, GetMenuItemsResponse, GetOrderCostResponse, GetOrderResponse,
        GetOrdersResponse, GetOwnersResponse, GetRestaurantsResponse, GetRiderResponse,
        GetUserOrdersResponse, GetUserRestaurantsResponse, OrderItem, PlatformConfigResponse,
    },
    state::{MenuItem, MENU_ITEMS, PLATFORM_CONFIG, RIDERS},
};

use cosmwasm_std::{Addr, Deps, StdError, StdResult, Uint128};

use crate::msg::GetOrderStatusResponse;

pub fn query_platform_config(deps: Deps) -> StdResult<PlatformConfigResponse> {
    let config = PLATFORM_CONFIG.load(deps.storage)?;
    let owner_address = config
        .owners
        .get(0)
        .map_or("".to_string(), |addr| addr.to_string());
    Ok(PlatformConfigResponse {
        platform_name: config.platform_name,
        platform_description: config.platform_description,
        owner_address,
        fee_percentage: config.fee_percentage,
        fee_address: config.fee_address.to_string(),
    })
}

pub fn get_owners(deps: Deps) -> StdResult<GetOwnersResponse> {
    let config = PLATFORM_CONFIG.load(deps.storage)?;
    let owners = config
        .owners
        .into_iter()
        .map(|addr| addr.to_string())
        .collect();
    Ok(GetOwnersResponse { owners })
}
pub fn get_all_restaurants(deps: Deps) -> StdResult<GetRestaurantsResponse> {
    let restaurants: Vec<Restaurant> = RESTAURANTS
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .collect::<StdResult<Vec<_>>>()?
        .into_iter()
        .map(|(_, restaurant)| restaurant)
        .collect();
    Ok(GetRestaurantsResponse { restaurants })
}

pub fn get_menu_items_for_restaurant(
    deps: Deps,
    restaurant_id: String,
) -> StdResult<GetMenuItemsResponse> {
    let menu_items: Vec<MenuItem> = MENU_ITEMS
        .prefix(&restaurant_id)
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .collect::<StdResult<Vec<_>>>()?
        .into_iter()
        .map(|(_, menu_item)| menu_item)
        .collect();
    Ok(GetMenuItemsResponse { menu_items })
}

pub fn get_orders_for_restaurant(
    deps: Deps,
    restaurant_id: String,
) -> StdResult<GetOrdersResponse> {
    let orders: Vec<Order> = ORDERS
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .filter_map(|item| {
            let (_, order) = item.ok()?;
            (order.restaurant_id == restaurant_id).then_some(order)
        })
        .collect();
    Ok(GetOrdersResponse { orders })
}

pub fn get_order_status(deps: Deps, is_delivered: bool) -> StdResult<Vec<Order>> {
    let orders: Vec<Order> = ORDERS
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .collect::<StdResult<Vec<_>>>()?
        .into_iter()
        .map(|(_, order)| order)
        .filter(|order| matches!(order.status, OrderStatus::Completed if is_delivered))
        .collect();
    Ok(orders)
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

pub fn get_rider(deps: Deps, rider_id: String) -> StdResult<GetRiderResponse> {
    let rider = RIDERS.may_load(deps.storage, &rider_id)?;
    Ok(GetRiderResponse { rider })
}

pub fn get_user_restaurants(deps: Deps, owner: Addr) -> StdResult<GetUserRestaurantsResponse> {
    let restaurants: Vec<Restaurant> = RESTAURANTS
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .filter_map(|item| {
            let (_, restaurant) = item.ok()?;
            (restaurant.owner == owner).then_some(restaurant)
        })
        .collect();
    Ok(GetUserRestaurantsResponse { restaurants })
}

pub fn get_rider_by_address(deps: Deps, address: Addr) -> StdResult<GetRiderResponse> {
    let rider_id = format!("rider_{}", address);
    let rider = RIDERS.may_load(deps.storage, &rider_id)?;
    Ok(GetRiderResponse { rider })
}

pub fn get_user_orders(deps: Deps, address: Addr) -> StdResult<GetUserOrdersResponse> {
    let orders: Vec<Order> = ORDERS
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .filter_map(|item| {
            let (_, order) = item.ok()?;
            (order.customer == address).then_some(order)
        })
        .collect();
    Ok(GetUserOrdersResponse { orders })
}

pub fn get_order_cost(
    deps: Deps,
    restaurant_id: String,
    items: Vec<OrderItem>,
) -> StdResult<GetOrderCostResponse> {
    if items.is_empty() {
        return Err(StdError::generic_err("Empty order"));
    }
    RESTAURANTS
        .load(deps.storage, &restaurant_id)
        .map_err(|_| StdError::generic_err("Restaurant not found"))?;
    let total = items.iter().fold(Ok(Uint128::zero()), |acc, item| {
        let acc = acc?;
        let menu_item = MENU_ITEMS
            .may_load(deps.storage, (&restaurant_id, &item.item_id))?
            .ok_or_else(|| StdError::generic_err("Item not found"))?;
        if !menu_item.available {
            return Err(StdError::generic_err("Item not available"));
        }
        let item_total = menu_item
            .price
            .checked_mul(Uint128::from(item.quantity))
            .map_err(|_| StdError::generic_err("Overflow in item total"))?;
        acc.checked_add(item_total)
            .map_err(|_| StdError::generic_err("Overflow in total"))
    })?;
    if total.is_zero() {
        return Err(StdError::generic_err("Invalid order amount"));
    }
    Ok(GetOrderCostResponse { total })
}
