#[cfg(not(feature = "library"))]
use crate::state::{Order, OrderStatus, Restaurant, ESCROWS, ORDERS, RESTAURANTS};
use crate::{
    msg::{
        GetEscrowResponse, GetMenuItemsResponse, GetOrderCostResponse, GetOrderResponse,
        GetOrdersResponse, GetRestaurantsResponse, GetRiderResponse, GetUserOrdersResponse,
        GetUserRestaurantsResponse, OrderItem, PlatformConfigResponse,
    },
    state::{MenuItem, Rider, MENU_ITEMS, PLATFORM_CONFIG, RIDERS},
};

use cosmwasm_std::{Addr, Deps, StdError, StdResult, Uint128};

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

pub fn get_user_restaurants(deps: Deps, owner: Addr) -> StdResult<GetUserRestaurantsResponse> {
    let restaurants: Vec<Restaurant> = RESTAURANTS
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .filter_map(|item| {
            let (_, restaurant) = item.unwrap();
            if restaurant.owner == owner {
                Some(restaurant)
            } else {
                None
            }
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
            let (_, order) = item.unwrap();
            if order.customer == address {
                Some(order)
            } else {
                None
            }
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
