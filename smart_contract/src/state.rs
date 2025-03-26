use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Decimal, Uint128};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::msg::OrderItem;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct MenuItem {
    pub id: String,
    pub name: String,
    pub price: Uint128,
    pub available: bool,
    pub image_uri: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
pub struct Restaurant {
    pub id: String,
    pub owner: Addr,
    pub name: String,
    pub image_uri: String,
    pub restaurant_address: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Order {
    pub id: String,
    pub customer: Addr,
    pub restaurant_id: String,
    pub items: Vec<OrderItem>,
    pub total: Uint128,
    pub status: OrderStatus,
    pub rider_id: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Rider {
    pub id: String,
    pub name: String,
    pub wallet: Addr,
    pub is_registered: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Escrow {
    pub order_id: String,
    pub amount: Uint128,
    pub released: bool,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema, Debug)]
pub enum OrderStatus {
    Created,
    Accepted,
    InDelivery,
    Completed,
    Cancelled,
}

#[cw_serde]
pub struct PlatformConfig {
    pub platform_name: String,
    pub platform_description: String,
    pub owner_address: Addr,
    pub fee_percentage: Decimal,
    pub fee_address: Addr,
}

pub const PLATFORM_CONFIG: Item<PlatformConfig> = Item::new("platform_config");
pub const RESTAURANTS: Map<&str, Restaurant> = Map::new("restaurants");
pub const MENU_ITEMS: Map<(&str, &str), MenuItem> = Map::new("menu_items");
pub const RIDERS: Map<&str, Rider> = Map::new("riders");
pub const ORDERS: Map<&str, Order> = Map::new("orders");
pub const ESCROWS: Map<&str, Escrow> = Map::new("escrows");
