use cosmwasm_schema::{cw_serde, QueryResponses};

use cosmwasm_std::{Addr, Decimal, Uint128};

use crate::state::{Escrow, MenuItem, Order, OrderStatus, Restaurant, Rider};

#[cw_serde]
#[derive(Eq)]
pub struct OrderItem {
    pub item_id: String,
    pub quantity: u32,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub platform_name: String,
    pub platform_description: String,
    pub owner_address: Addr,
    pub fee_percentage: Decimal,
    pub fee_address: Addr,
}

#[cw_serde]
pub enum ExecuteMsg {
    RegisterRestaurant {
        name: String,
        image_uri: String,
        restaurant_address: Addr,
    },
    RegisterRider {
        name: String,
    },
    AddMenuItem {
        item_id: String,
        name: String,
        price: Uint128,
        image_uri: String,
    },
    CreateOrder {
        restaurant_id: String,
        items: Vec<OrderItem>,
    },
    AcceptOrder {
        order_id: String,
    },
    AssignRider {
        order_id: String,
        rider_id: String,
    },
    ConfirmDelivery {
        order_id: String,
    },
    DepositFunds {
        order_id: String,
    },
    ReleaseFunds {
        order_id: String,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(GetRestaurantsResponse)]
    GetRestaurants {},

    #[returns(GetMenuItemsResponse)]
    GetMenuItems { restaurant_id: String },

    #[returns(GetOrdersResponse)]
    GetOrdersFromARestaurant { restaurant_id: String },

    #[returns(GetOrderResponse)]
    GetOrder { id: String },

    #[returns(GetOrderStatus)]
    GetAllSuccessfulOrderStatus { is_delivered: bool },

    #[returns(GetOrderStatusResponse)]
    GetOrderStatusById { order_id: String },

    #[returns(Escrow)]
    GetEscrow { order_id: String },

    #[returns(PlatformConfigResponse)]
    GetPlatformConfig {},

    #[returns(Rider)]
    GetRider { rider_id: String },

    #[returns(GetRiderResponse)]
    GetRiderByAddress { riders_address: String },

    #[returns(GetUserRestaurantsResponse)]
    GetUserRestaurants { owner: String },

    #[returns(GetUserOrdersResponse)]
    GetOrders { address: String },
}

#[cw_serde]
pub struct GetRiderResponse {
    pub rider: Option<Rider>,
}

#[cw_serde]
pub struct GetRestaurantsResponse {
    pub restaurants: Vec<Restaurant>,
}

#[cw_serde]
pub struct GetUserRestaurantsResponse {
    pub restaurants: Vec<Restaurant>,
}

#[cw_serde]
pub struct GetOrderStatus {
    pub order: Order,
}

#[cw_serde]
pub struct GetOrderResponse {
    pub order: Order,
}

#[cw_serde]
pub struct GetEscrowResponse {
    pub escrow: Escrow,
}

#[cw_serde]
pub struct GetMenuItemsResponse {
    pub menu_items: Vec<MenuItem>,
}

#[cw_serde]
pub struct GetOrdersResponse {
    pub orders: Vec<Order>,
}

#[cw_serde]
pub struct GetUserOrdersResponse {
    pub orders: Vec<Order>,
}

#[cw_serde]
pub struct GetOrderStatusResponse {
    pub order_id: String,
    pub status: OrderStatus,
}

#[cw_serde]
pub struct PlatformConfigResponse {
    pub platform_name: String,
    pub platform_description: String,
    pub owner_address: Addr,
    pub fee_percentage: Decimal,
    pub fee_address: Addr,
}
