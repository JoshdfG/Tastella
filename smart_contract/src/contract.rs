#[cfg(not(feature = "library"))]
use crate::error::ContractError;
use crate::execute::{self, init};
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::query::{
    get_all_restaurants, get_escrow, get_menu_items_for_restaurant, get_order_by_id,
    get_order_status, get_order_status_by_id, get_orders_for_restaurant, get_rider,
    get_rider_by_address, get_user_orders, get_user_restaurants, query_platform_config,
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

        ExecuteMsg::AcceptOrder { order_id } => execute::accept_order(deps, info, order_id),

        ExecuteMsg::DepositFunds { order_id } => execute::deposit_funds(deps, info, order_id),

        ExecuteMsg::ReleaseFunds { order_id } => execute::release_funds(deps, order_id),

        ExecuteMsg::AssignRider { order_id, rider_id } => {
            execute::assign_rider(deps, order_id, rider_id)
        }

        ExecuteMsg::ConfirmDelivery { order_id } => {
            execute::confirm_delivery(deps, env, info, order_id)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetPlatformConfig {} => to_json_binary(&query_platform_config(deps)?),

        QueryMsg::GetRestaurants {} => to_json_binary(&get_all_restaurants(deps)?),

        QueryMsg::GetAllSuccessfulOrderStatus { is_delivered } => {
            to_json_binary(&get_order_status(deps, is_delivered)?)
        }

        QueryMsg::GetOrder { id } => to_json_binary(&get_order_by_id(deps, id)?),

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

        QueryMsg::GetRider { rider_id } => to_json_binary(&get_rider(deps, rider_id)?),

        QueryMsg::GetRiderByAddress { riders_address } => {
            to_json_binary(&get_rider_by_address(deps, riders_address)?)
        }

        QueryMsg::GetUserRestaurants { owner } => {
            to_json_binary(&get_user_restaurants(deps, owner)?)
        }

        QueryMsg::GetOrders { address } => {
            let response = get_user_orders(deps, address)?;
            to_json_binary(&response)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::msg::{GetOrdersResponse, GetRestaurantsResponse, GetRiderResponse, OrderItem};
    use crate::state::{
        OrderStatus, PlatformConfig, Restaurant, MENU_ITEMS, ORDERS, PLATFORM_CONFIG, RESTAURANTS,
        RIDERS,
    };
    use cosmwasm_std::testing::{
        mock_dependencies, mock_env, mock_info, MockApi, MockQuerier, MockStorage,
    };
    use cosmwasm_std::{attr, coins, from_json, Addr, Coin, Decimal, OwnedDeps, Uint128};

    fn setup_contract() -> (
        OwnedDeps<MockStorage, MockApi, MockQuerier>,
        Env,
        MessageInfo,
        Addr,
    ) {
        let mut deps = mock_dependencies();

        let env = mock_env();

        let info = mock_info("creator", &[]);

        let restaurant_address = Addr::unchecked("res");

        let platform_config = PlatformConfig {
            platform_name: "Food Delivery Platform".to_string(),
            platform_description: "A decentralized food delivery platform".to_string(),
            owner_address: info.sender.clone(),
            fee_percentage: Decimal::percent(5),
            fee_address: Addr::unchecked("fee_wallet"),
        };
        PLATFORM_CONFIG
            .save(&mut deps.storage, &platform_config)
            .unwrap();
        (deps, env, info, restaurant_address)
    }

    #[test]
    fn test_register_restaurant() {
        let (mut deps, env, info, restaurant_address) = setup_contract();

        let res = execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            ExecuteMsg::RegisterRestaurant {
                name: "Test Restaurant".to_string(),
                image_uri: "https://test.com".to_string(),
                restaurant_address,
            },
        )
        .unwrap();

        assert_eq!(res.messages.len(), 0);
        assert_eq!(res.attributes, vec![attr("action", "register_restaurant")]);

        let restaurant = RESTAURANTS
            .load(&deps.storage, "restaurant_creator")
            .unwrap();
        assert_eq!(restaurant.name, "Test Restaurant");
        assert_eq!(restaurant.owner, Addr::unchecked("creator"));
    }

    #[test]
    fn test_add_menu_item() {
        let (mut deps, env, info, restaurant_address) = setup_contract();
        let msg = ExecuteMsg::RegisterRestaurant {
            name: "Test Restaurant".to_string(),
            image_uri: "https://test.com".to_string(),
            restaurant_address,
        };
        execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let res = execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            ExecuteMsg::AddMenuItem {
                item_id: "item_1".to_string(),
                name: "Pizza".to_string(),
                price: Uint128::from(100u128),
                image_uri: "https://test.com/pizza".to_string(),
            },
        )
        .unwrap();

        assert_eq!(res.messages.len(), 0);
        assert_eq!(res.attributes, vec![attr("action", "add_menu_item")]);

        let menu_item = MENU_ITEMS
            .load(&deps.storage, ("restaurant_creator", "item_1"))
            .unwrap();
        assert_eq!(menu_item.name, "Pizza");
        assert_eq!(menu_item.price, Uint128::from(100u128));
    }

    #[test]
    fn test_create_order() {
        let (mut deps, env, info, restaurant_address) = setup_contract();

        execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            ExecuteMsg::RegisterRestaurant {
                name: "Test Restaurant".to_string(),
                image_uri: "https://test.com".to_string(),
                restaurant_address,
            },
        )
        .unwrap();

        execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            ExecuteMsg::AddMenuItem {
                item_id: "item_1".to_string(),
                name: "Pizza".to_string(),
                price: Uint128::from(100u128),
                image_uri: "https://test.com/pizza".to_string(),
            },
        )
        .unwrap();

        let res = execute(
            deps.as_mut(),
            env.clone(),
            mock_info("customer", &coins(200, "uxion")),
            ExecuteMsg::CreateOrder {
                restaurant_id: "restaurant_creator".to_string(),
                items: vec![OrderItem {
                    item_id: "item_1".to_string(),
                    quantity: 2,
                }],
            },
        )
        .unwrap();

        assert_eq!(res.messages.len(), 0);
        assert_eq!(
            res.attributes,
            vec![
                attr("action", "create_order"),
                attr("order_id", "order_1"),
                attr("restaurant_id", "restaurant_creator"),
                attr("total", "200")
            ]
        );

        let order = ORDERS.load(&deps.storage, "order_1").unwrap();
        assert_eq!(order.customer, Addr::unchecked("customer"));
        assert_eq!(order.total, Uint128::from(200u128));
    }

    #[test]
    fn test_accept_order() {
        let (mut deps, env, info, restaurant_address) = setup_contract();

        execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            ExecuteMsg::RegisterRestaurant {
                name: "Test Restaurant".to_string(),
                image_uri: "https://test.com".to_string(),
                restaurant_address,
            },
        )
        .unwrap();

        execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            ExecuteMsg::AddMenuItem {
                item_id: "item_1".to_string(),
                name: "Pizza".to_string(),
                price: Uint128::from(100u128),
                image_uri: "https://test.com/pizza".to_string(),
            },
        )
        .unwrap();

        execute(
            deps.as_mut(),
            env.clone(),
            mock_info("customer", &coins(200, "uxion")),
            ExecuteMsg::CreateOrder {
                restaurant_id: "restaurant_creator".to_string(),
                items: vec![OrderItem {
                    item_id: "item_1".to_string(),
                    quantity: 2,
                }],
            },
        )
        .unwrap();

        let res = execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            ExecuteMsg::AcceptOrder {
                order_id: "order_1".to_string(),
            },
        )
        .unwrap();

        assert_eq!(res.messages.len(), 0);
        assert_eq!(
            res.attributes,
            vec![attr("action", "accept_order"), attr("order_id", "order_1"),]
        );

        let order = ORDERS.load(&deps.storage, "order_1").unwrap();
        assert_eq!(order.status, OrderStatus::Accepted);
    }
    #[test]
    fn test_register_rider() {
        let (mut deps, env, _info, _restaurant_address) = setup_contract();

        let res = execute(
            deps.as_mut(),
            env.clone(),
            mock_info("rider", &[]),
            ExecuteMsg::RegisterRider {
                name: "Test Rider".to_string(),
            },
        )
        .unwrap();

        assert_eq!(res.messages.len(), 0);
        assert_eq!(
            res.attributes,
            vec![
                attr("action", "register_rider"),
                attr("rider_id", "rider_rider")
            ]
        );

        let rider = RIDERS.load(&deps.storage, "rider_rider").unwrap();
        assert_eq!(rider.name, "Test Rider");
        assert_eq!(rider.wallet, Addr::unchecked("rider"));
    }
    #[test]
    fn test_assign_rider() {
        let (mut deps, env, info, restaurant_address) = setup_contract();

        execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            ExecuteMsg::RegisterRestaurant {
                name: "Test Restaurant".to_string(),
                image_uri: "https://test.com".to_string(),
                restaurant_address,
            },
        )
        .unwrap();

        execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            ExecuteMsg::AddMenuItem {
                item_id: "item_1".to_string(),
                name: "Pizza".to_string(),
                price: Uint128::from(100u128),
                image_uri: "https://test.com/pizza".to_string(),
            },
        )
        .unwrap();

        execute(
            deps.as_mut(),
            env.clone(),
            mock_info("customer", &coins(200, "uxion")),
            ExecuteMsg::CreateOrder {
                restaurant_id: "restaurant_creator".to_string(),
                items: vec![OrderItem {
                    item_id: "item_1".to_string(),
                    quantity: 2,
                }],
            },
        )
        .unwrap();

        execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            ExecuteMsg::AcceptOrder {
                order_id: "order_1".to_string(),
            },
        )
        .unwrap();

        execute(
            deps.as_mut(),
            env.clone(),
            mock_info("rider", &[]),
            ExecuteMsg::RegisterRider {
                name: "Test Rider".to_string(),
            },
        )
        .unwrap();

        let res = execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            ExecuteMsg::AssignRider {
                order_id: "order_1".to_string(),
                rider_id: "rider_rider".to_string(),
            },
        )
        .unwrap();

        assert_eq!(res.messages.len(), 0);
        assert_eq!(
            res.attributes,
            vec![
                attr("action", "assign_rider"),
                attr("order_id", "order_1"),
                attr("rider_id", "rider_rider")
            ]
        );

        let order = ORDERS.load(&deps.storage, "order_1").unwrap();
        assert_eq!(order.status, OrderStatus::InDelivery);
        assert_eq!(order.rider_id, Some("rider_rider".to_string()));
    }
    #[test]
    fn test_confirm_delivery() {
        let (mut deps, env, info, restaurant_address) = setup_contract();

        execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            ExecuteMsg::RegisterRestaurant {
                name: "Test Restaurant".to_string(),
                image_uri: "https://test.com".to_string(),
                restaurant_address,
            },
        )
        .unwrap();

        execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            ExecuteMsg::AddMenuItem {
                item_id: "item_1".to_string(),
                name: "Pizza".to_string(),
                price: Uint128::from(100u128),
                image_uri: "https://test.com/pizza".to_string(),
            },
        )
        .unwrap();

        let customer_info = mock_info("customer", &coins(200, "uxion"));
        execute(
            deps.as_mut(),
            env.clone(),
            customer_info.clone(),
            ExecuteMsg::CreateOrder {
                restaurant_id: "restaurant_creator".to_string(),
                items: vec![OrderItem {
                    item_id: "item_1".to_string(),
                    quantity: 2,
                }],
            },
        )
        .unwrap();

        deps.querier.update_balance(
            env.contract.address.clone(),
            vec![Coin {
                denom: "uxion".to_string(),
                amount: Uint128::new(200),
            }],
        );

        execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            ExecuteMsg::AcceptOrder {
                order_id: "order_1".to_string(),
            },
        )
        .unwrap();

        execute(
            deps.as_mut(),
            env.clone(),
            mock_info("rider", &[]),
            ExecuteMsg::RegisterRider {
                name: "Test Rider".to_string(),
            },
        )
        .unwrap();

        // Assign rider
        execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            ExecuteMsg::AssignRider {
                order_id: "order_1".to_string(),
                rider_id: "rider_rider".to_string(),
            },
        )
        .unwrap();

        let res = execute(
            deps.as_mut(),
            env.clone(),
            mock_info("rider", &[]),
            ExecuteMsg::ConfirmDelivery {
                order_id: "order_1".to_string(),
            },
        )
        .unwrap();

        assert_eq!(res.messages.len(), 2);
        assert_eq!(
            res.attributes,
            vec![
                attr("action", "confirm_delivery"),
                attr("order_id", "order_1"),
                attr("status", "Completed")
            ]
        );

        let order = ORDERS.load(&deps.storage, "order_1").unwrap();
        assert_eq!(order.status, OrderStatus::Completed);
    }

    #[test]
    fn test_get_restaurants() {
        let (mut deps, env, info, restaurant_address) = setup_contract();

        execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            ExecuteMsg::RegisterRestaurant {
                name: "Test Restaurant".to_string(),
                image_uri: "https://test.com".to_string(),
                restaurant_address,
            },
        )
        .unwrap();

        let res = query(deps.as_ref(), env.clone(), QueryMsg::GetRestaurants {}).unwrap();

        let response: GetRestaurantsResponse = from_json(&res).unwrap();

        let restaurants = response.restaurants;
        assert_eq!(restaurants.len(), 1);
        assert_eq!(restaurants[0].name, "Test Restaurant");
    }

    #[test]
    fn test_get_multiple_restaurants() {
        let (mut deps, env, _restaurant_address, _) = setup_contract();

        let restaurants = vec![
            Restaurant {
                id: "restaurant_1".to_string(),
                owner: Addr::unchecked("owner_1"),
                name: "Restaurant One".to_string(),
                image_uri: "https://test.com/restaurant1".to_string(),
                restaurant_address: Addr::unchecked("q"),
            },
            Restaurant {
                id: "restaurant_2".to_string(),
                owner: Addr::unchecked("owner_2"),
                name: "Restaurant Two".to_string(),
                image_uri: "https://test.com/restaurant2".to_string(),
                restaurant_address: Addr::unchecked("c"),
            },
            Restaurant {
                id: "restaurant_3".to_string(),
                owner: Addr::unchecked("owner_3"),
                name: "Restaurant Three".to_string(),
                image_uri: "https://test.com/restaurant3".to_string(),
                restaurant_address: Addr::unchecked("a"),
            },
        ];

        for restaurant in restaurants {
            execute(
                deps.as_mut(),
                env.clone(),
                mock_info(restaurant.owner.as_str(), &[]),
                ExecuteMsg::RegisterRestaurant {
                    name: restaurant.name.clone(),
                    image_uri: restaurant.image_uri.clone(),
                    restaurant_address: Addr::unchecked("q"),
                },
            )
            .unwrap();
        }

        let res = query(deps.as_ref(), env.clone(), QueryMsg::GetRestaurants {}).unwrap();
        let response: GetRestaurantsResponse = from_json(&res).unwrap();
        let retrieved_restaurants = response.restaurants;

        assert_eq!(retrieved_restaurants.len(), 3);
        assert_eq!(retrieved_restaurants[0].name, "Restaurant One");
        assert_eq!(retrieved_restaurants[1].name, "Restaurant Two");
        assert_eq!(retrieved_restaurants[2].name, "Restaurant Three");
    }

    #[test]
    fn test_get_a_restaurants_menu() {
        let (mut deps, env, info, restaurant_address) = setup_contract();
        let msg = ExecuteMsg::RegisterRestaurant {
            name: "Test Restaurant".to_string(),
            image_uri: "https://test.com".to_string(),
            restaurant_address,
        };
        execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let res = execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            ExecuteMsg::AddMenuItem {
                item_id: "item_1".to_string(),
                name: "Pizza".to_string(),
                price: Uint128::from(100u128),
                image_uri: "https://test.com/pizza".to_string(),
            },
        )
        .unwrap();

        assert_eq!(res.messages.len(), 0);
        assert_eq!(res.attributes, vec![attr("action", "add_menu_item")]);

        let menu_item = MENU_ITEMS
            .load(&deps.storage, ("restaurant_creator", "item_1"))
            .unwrap();
        assert_eq!(menu_item.name, "Pizza");
        assert_eq!(menu_item.price, Uint128::from(100u128));
    }

    #[test]
    fn test_get_rider() {
        let (mut deps, env, _info, _restaurant_address) = setup_contract();

        execute(
            deps.as_mut(),
            env.clone(),
            mock_info("rider", &[]),
            ExecuteMsg::RegisterRider {
                name: "Test Rider".to_string(),
            },
        )
        .unwrap();

        let res: GetRiderResponse = from_json(
            query(
                deps.as_ref(),
                env.clone(),
                QueryMsg::GetRiderByAddress {
                    riders_address: "rider".to_string(),
                },
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(res.rider.unwrap().id, "rider_rider");
    }

    #[test]
    fn test_get_user_restaurants() {
        let (mut deps, env, info, restaurant_address) = setup_contract();

        execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            ExecuteMsg::RegisterRestaurant {
                name: "Test Restaurant".to_string(),
                image_uri: "https://test.com".to_string(),
                restaurant_address,
            },
        )
        .unwrap();

        let res: GetRestaurantsResponse = from_json(
            query(
                deps.as_ref(),
                env.clone(),
                QueryMsg::GetUserRestaurants {
                    owner: "creator".to_string(),
                },
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(res.restaurants.len(), 1);
        assert_eq!(res.restaurants[0].id, "restaurant_creator");
    }
    #[test]
    fn test_get_orders() {
        let (mut deps, env, info, restaurant_address) = setup_contract();

        execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            ExecuteMsg::RegisterRestaurant {
                name: "Test Restaurant".to_string(),
                image_uri: "https://test.com".to_string(),
                restaurant_address,
            },
        )
        .unwrap();

        execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            ExecuteMsg::AddMenuItem {
                item_id: "item_1".to_string(),
                name: "Pizza".to_string(),
                price: Uint128::from(100u128),
                image_uri: "https://test.com/pizza".to_string(),
            },
        )
        .unwrap();

        execute(
            deps.as_mut(),
            env.clone(),
            mock_info("customer", &coins(200, "uxion")),
            ExecuteMsg::CreateOrder {
                restaurant_id: "restaurant_creator".to_string(),
                items: vec![OrderItem {
                    item_id: "item_1".to_string(),
                    quantity: 2,
                }],
            },
        )
        .unwrap();

        let res: GetOrdersResponse = from_json(
            query(
                deps.as_ref(),
                env.clone(),
                QueryMsg::GetOrders {
                    address: "customer".to_string(),
                },
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(res.orders.len(), 1);
        assert_eq!(res.orders[0].id, "order_1");
        assert_eq!(res.orders[0].customer, Addr::unchecked("customer"));
        assert_eq!(res.orders[0].total, Uint128::new(200));
    }
}
