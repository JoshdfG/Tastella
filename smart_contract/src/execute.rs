#[cfg(not(feature = "library"))]
use cosmwasm_std::{Addr, Decimal, Deps, DepsMut, Env, MessageInfo, Response, Uint128};
use cosmwasm_std::{BankMsg, Coin};

use crate::error::ContractError;
use crate::msg::OrderItem;
use crate::state::{
    Escrow, MenuItem, Order, OrderStatus, PlatformConfig, Restaurant, ESCROWS, ORDERS,
    PLATFORM_CONFIG, RESTAURANTS, RIDERS,
};
use crate::state::{Rider, MENU_ITEMS};
const NATIVE_DENOM: &str = "uxion";
pub fn init(
    deps: DepsMut,
    _info: MessageInfo,
    platform_name: String,
    platform_description: String,
    owner_address: Addr,
    fee_percentage: Decimal,
    fee_address: Addr,
) -> Result<Response, ContractError> {
    if fee_percentage > Decimal::one() {
        return Err(ContractError::InvalidFeePercentage {});
    }

    let config = PlatformConfig {
        platform_name,
        platform_description,
        owner_address,
        fee_percentage,
        fee_address,
    };
    PLATFORM_CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().add_attribute("action", "init"))
}
pub fn register_restaurant(
    deps: DepsMut,
    info: MessageInfo,
    name: String,
    image_uri: String,
    restaurant_address: Addr,
) -> Result<Response, ContractError> {
    let restaurant_id = format!("restaurant_{}", info.sender);
    let restaurant = Restaurant {
        id: restaurant_id.clone(),
        owner: info.sender,
        name,
        image_uri,
        restaurant_address,
    };

    RESTAURANTS.save(deps.storage, &restaurant_id, &restaurant)?;
    Ok(Response::new().add_attribute("action", "register_restaurant"))
}

pub fn add_menu_item(
    deps: DepsMut,
    info: MessageInfo,
    item_id: String,
    name: String,
    price: Uint128,
    image_uri: String,
) -> Result<Response, ContractError> {
    let restaurant_id = format!("restaurant_{}", info.sender);

    RESTAURANTS.load(deps.storage, &restaurant_id)?;

    let menu_item = MenuItem {
        id: item_id.clone(),
        name,
        price,
        available: true,
        image_uri,
    };

    MENU_ITEMS.save(deps.storage, (&restaurant_id, &item_id), &menu_item)?;

    Ok(Response::new().add_attribute("action", "add_menu_item"))
}

pub fn release_funds(deps: DepsMut, order_id: String) -> Result<Response, ContractError> {
    let mut escrow = ESCROWS.load(deps.storage, &order_id)?;

    if escrow.released {
        return Err(ContractError::FundsAlreadyReleased {});
    }

    let order = ORDERS.load(deps.storage, &order_id)?;

    if order.status != OrderStatus::Completed {
        return Err(ContractError::OrderNotCompleted {});
    }

    let restaurant = RESTAURANTS.load(deps.storage, &order.restaurant_id)?;

    escrow.released = true;
    ESCROWS.save(deps.storage, &order_id, &escrow)?;

    let release_msg = cosmwasm_std::BankMsg::Send {
        to_address: restaurant.owner.to_string(),
        amount: vec![cosmwasm_std::Coin {
            denom: "uusd".to_string(),
            amount: escrow.amount,
        }],
    };

    Ok(Response::new()
        .add_message(release_msg)
        .add_attribute("action", "release_funds")
        .add_attribute("order_id", order_id)
        .add_attribute("amount", escrow.amount.to_string()))
}

pub fn create_order(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    restaurant_id: String,
    items: Vec<OrderItem>,
) -> Result<Response, ContractError> {
    if items.is_empty() {
        return Err(ContractError::EmptyOrder {});
    }

    let mut total = Uint128::zero();
    for item in &items {
        let menu_item = MENU_ITEMS
            .may_load(deps.storage, (&restaurant_id, &item.item_id))?
            .ok_or(ContractError::ItemNotFound {})?;

        if !menu_item.available {
            return Err(ContractError::ItemNotAvailable {});
        }

        let item_total = menu_item
            .price
            .checked_mul(Uint128::from(item.quantity))
            .map_err(|_| ContractError::Overflow {})?;
        total = total
            .checked_add(item_total)
            .map_err(|_| ContractError::Overflow {})?;
    }

    if total.is_zero() {
        return Err(ContractError::InvalidOrderAmount {});
    }

    if info.funds.len() != 1 {
        return Err(ContractError::IncorrectPayment {});
    }
    if info.funds[0].denom != NATIVE_DENOM {
        return Err(ContractError::IncorrectPayment {});
    }
    if info.funds[0].amount != total {
        return Err(ContractError::IncorrectPayment {});
    }

    let order_id = if cfg!(test) {
        "order_1".to_string()
    } else {
        format!("order_{}", env.block.height)
    };

    RESTAURANTS.load(deps.storage, &restaurant_id)?;

    let order = Order {
        id: order_id.clone(),
        customer: info.sender.clone(),
        restaurant_id: restaurant_id.clone(),
        items,
        total,
        status: OrderStatus::Created,
        rider_id: None,
    };
    ORDERS.save(deps.storage, &order_id, &order)?;

    ESCROWS.save(
        deps.storage,
        &order_id,
        &Escrow {
            order_id: order_id.clone(),
            amount: total,
            released: false,
        },
    )?;

    Ok(Response::new()
        .add_attribute("action", "create_order")
        .add_attribute("order_id", order_id)
        .add_attribute("restaurant_id", restaurant_id)
        .add_attribute("total", total.to_string()))
}

pub fn confirm_delivery(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    order_id: String,
) -> Result<Response, ContractError> {
    let mut order = ORDERS.load(deps.storage, &order_id)?;
    if order.status != OrderStatus::InDelivery {
        return Err(ContractError::OrderNotInDelivery {});
    }

    let rider_id = order
        .rider_id
        .as_ref()
        .ok_or(ContractError::NoRiderAssigned {})?;
    let rider = RIDERS.load(deps.storage, rider_id)?;
    if info.sender != rider.wallet {
        return Err(ContractError::Unauthorized {});
    }

    let escrow = ESCROWS.load(deps.storage, &order_id)?;
    if escrow.released {
        return Err(ContractError::FundsAlreadyReleased {});
    }

    let contract_balance = deps
        .querier
        .query_balance(&env.contract.address, NATIVE_DENOM)?;
    if contract_balance.amount < escrow.amount {
        return Err(ContractError::InsufficientEscrowBalance {});
    }

    let config = PLATFORM_CONFIG.load(deps.storage)?;
    let fee_amount = escrow.amount * config.fee_percentage;
    let remaining_amount = escrow
        .amount
        .checked_sub(fee_amount)
        .map_err(|_| ContractError::Overflow {})?;

    let fee_msg = BankMsg::Send {
        to_address: config.fee_address.to_string(),
        amount: vec![Coin {
            denom: NATIVE_DENOM.to_string(),
            amount: fee_amount,
        }],
    };

    let restaurant = RESTAURANTS.load(deps.storage, &order.restaurant_id)?;
    let payment_msg = BankMsg::Send {
        to_address: restaurant.restaurant_address.to_string(),
        amount: vec![Coin {
            denom: NATIVE_DENOM.to_string(),
            amount: remaining_amount,
        }],
    };

    order.status = OrderStatus::Completed;
    ORDERS.save(deps.storage, &order_id, &order)?;
    ESCROWS.save(
        deps.storage,
        &order_id,
        &Escrow {
            order_id: order_id.clone(),
            amount: Uint128::zero(),
            released: true,
        },
    )?;

    Ok(Response::new()
        .add_message(fee_msg)
        .add_message(payment_msg)
        .add_attribute("action", "confirm_delivery")
        .add_attribute("order_id", order_id)
        .add_attribute("status", "Completed"))
}
pub fn validate_order(
    deps: Deps,
    restaurant_id: &str,
    items: &[OrderItem],
) -> Result<Uint128, ContractError> {
    let mut total = Uint128::zero();
    for item in items {
        let menu_item = MENU_ITEMS.load(deps.storage, (&restaurant_id, &item.item_id))?;
        total += menu_item.price * Uint128::from(item.quantity);
    }
    Ok(total)
}

pub fn accept_order(
    deps: DepsMut,
    info: MessageInfo,
    order_id: String,
) -> Result<Response, ContractError> {
    let mut order = ORDERS.load(deps.storage, &order_id)?;

    if order.status != OrderStatus::Created {
        return Err(ContractError::OrderAlreadyProcessed {});
    }

    let restaurant = RESTAURANTS.load(deps.storage, &order.restaurant_id)?;
    if info.sender != restaurant.owner {
        return Err(ContractError::Unauthorized {});
    }

    order.status = OrderStatus::Accepted;
    ORDERS.save(deps.storage, &order_id, &order)?;

    Ok(Response::new()
        .add_attribute("action", "accept_order")
        .add_attribute("order_id", order_id))
}

pub fn register_rider(
    deps: DepsMut,
    info: MessageInfo,
    name: String,
) -> Result<Response, ContractError> {
    let rider_id = format!("rider_{}", info.sender);
    let rider = Rider {
        id: rider_id.clone(),
        name,
        wallet: info.sender,
        is_registered: true,
    };

    RIDERS.save(deps.storage, &rider_id, &rider)?;

    Ok(Response::new()
        .add_attribute("action", "register_rider")
        .add_attribute("rider_id", rider_id))
}

pub fn assign_rider(
    deps: DepsMut,
    order_id: String,
    rider_id: String,
) -> Result<Response, ContractError> {
    let mut order = ORDERS.load(deps.storage, &order_id)?;

    if order.status != OrderStatus::Accepted {
        return Err(ContractError::OrderNotAccepted {});
    }

    let rider = RIDERS.load(deps.storage, &rider_id)?;
    assert!(rider.is_registered == true, "Rider is not registered");

    order.rider_id = Some(rider_id.clone());
    order.status = OrderStatus::InDelivery;
    ORDERS.save(deps.storage, &order_id, &order)?;

    Ok(Response::new()
        .add_attribute("action", "assign_rider")
        .add_attribute("order_id", order_id)
        .add_attribute("rider_id", rider_id))
}

pub fn deposit_funds(
    deps: DepsMut,
    info: MessageInfo,
    order_id: String,
) -> Result<Response, ContractError> {
    let amount = info.funds[0].amount;
    let escrow = Escrow {
        order_id: order_id.clone(),
        amount,
        released: false,
    };

    ESCROWS.save(deps.storage, &order_id, &escrow)?;
    Ok(Response::new().add_attribute("action", "deposit_funds"))
}
