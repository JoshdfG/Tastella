#[cfg(test)]
mod tests {
    use tastella::contract::{execute, instantiate, query};
    use tastella::msg::{
        ExecuteMsg, GetEscrowResponse, GetRestaurantsResponse, InstantiateMsg, OrderItem, QueryMsg,
    };

    use cosmwasm_std::{Addr, Coin, Decimal, Empty, Uint128};
    use cw_multi_test::{App, AppBuilder, AppResponse, Contract, ContractWrapper, Executor};
    use lazy_static::lazy_static;
    use tastella::state::OrderStatus;

    pub fn contract_template() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(execute, instantiate, query);
        Box::new(contract)
    }

    const USER: &str = "xion1useraddress";
    const USER_2: &str = "xion1adminaddress";
    const NATIVE_DENOM: &str = "uxion";

    lazy_static! {
        static ref RESTAURANT_1: Addr = Addr::unchecked("xion1restaurant1");
        static ref RESTAURANT_2: Addr = Addr::unchecked("xion1restaurant2");
        static ref FEE_WALLET: Addr = Addr::unchecked("xion1fee_wallet");
    }

    fn mock_app() -> App {
        AppBuilder::new().build(|router, _, storage| {
            router
                .bank
                .init_balance(
                    storage,
                    &Addr::unchecked(USER),
                    vec![Coin {
                        denom: NATIVE_DENOM.to_string(),
                        amount: Uint128::new(100000),
                    }],
                )
                .unwrap();
        })
    }

    fn proper_instantiate() -> (App, Addr) {
        let mut app = mock_app();
        let contract_id = app.store_code(contract_template());

        let msg = InstantiateMsg {
            platform_name: "Food Delivery Platform".to_string(),
            platform_description: "A decentralized food delivery platform".to_string(),
            owner_address: "xion1adminaddress".to_string(),
            fee_percentage: Decimal::percent(5),
            fee_address: FEE_WALLET.to_string(),
        };

        let contract_addr = app
            .instantiate_contract(
                contract_id,
                Addr::unchecked("xion1adminaddress"),
                &msg,
                &[],
                "Restaurant Contract",
                None,
            )
            .unwrap();

        (app, contract_addr)
    }

    fn register_restaurant(
        app: &mut App,
        contract_addr: &Addr,
        user: &str,
        name: &str,
        image_uri: &str,
        restaurant_address: Addr,
    ) {
        let register_msg = ExecuteMsg::RegisterRestaurant {
            name: name.to_string(),
            image_uri: image_uri.to_string(),
            restaurant_address: restaurant_address.to_string(),
        };
        app.execute_contract(
            Addr::unchecked(user),
            contract_addr.clone(),
            &register_msg,
            &[],
        )
        .unwrap();
    }

    fn register_rider(app: &mut App, contract_addr: &Addr, user: &str, name: String) {
        let register_rider_msg = ExecuteMsg::RegisterRider {
            name: name.to_string(),
            phone_number: "1234567890".to_string(),
        };
        app.execute_contract(
            Addr::unchecked(user),
            contract_addr.clone(),
            &register_rider_msg,
            &[],
        )
        .unwrap();
    }

    fn add_menu_item(
        app: &mut App,
        contract_addr: &Addr,
        user: &str,
        _restaurant_id: &str,
        item_id: &str,
        name: &str,
        price: Uint128,
        image_uri: &str,
    ) {
        let add_menu_item_msg = ExecuteMsg::AddMenuItem {
            item_id: item_id.to_string(),
            name: name.to_string(),
            price,
            image_uri: image_uri.to_string(),
        };
        app.execute_contract(
            Addr::unchecked(user),
            contract_addr.clone(),
            &add_menu_item_msg,
            &[],
        )
        .unwrap();
    }

    fn create_order(
        app: &mut App,
        contract_addr: &Addr,
        user: &str,
        restaurant_id: &str,
        items: Vec<OrderItem>,
        funds: Vec<Coin>,
    ) -> Result<AppResponse, anyhow::Error> {
        let create_order_msg = ExecuteMsg::CreateOrder {
            restaurant_id: restaurant_id.to_string(),
            items,
        };
        app.execute_contract(
            Addr::unchecked(user),
            contract_addr.clone(),
            &create_order_msg,
            &funds,
        )
        .map_err(|e| anyhow::anyhow!(e))
    }

    mod restaurant_tests {

        use tastella::msg::{
            GetMenuItemsResponse, GetOrderCostResponse, GetOrderStatusResponse, GetOrdersResponse,
            GetOwnersResponse, GetRiderResponse, OrderItem, UserResponse,
        };

        use super::*;

        #[test]
        fn test_get_owners() {
            let (mut app, contract_addr) = proper_instantiate();

            app.execute_contract(
                Addr::unchecked(USER_2),
                contract_addr.clone(),
                &ExecuteMsg::AddNewOwner {
                    new_owner: "xion1newowner".to_string(),
                },
                &[],
            )
            .unwrap();

            let query_msg = QueryMsg::GetOwners {};
            let res: GetOwnersResponse = app
                .wrap()
                .query_wasm_smart(contract_addr.clone(), &query_msg)
                .unwrap();

            assert_eq!(res.owners.len(), 2);
            assert!(res.owners.contains(&"xion1adminaddress".to_string()));
            assert!(res.owners.contains(&"xion1newowner".to_string()));
        }

        #[test]
        fn test_register_and_get_restaurants() {
            let (mut app, contract_addr) = proper_instantiate();

            register_restaurant(
                &mut app,
                &contract_addr,
                USER,
                "Test Restaurant",
                "https://ipfs.io/ipfs/QmXoypizjW3WknFiJnKLwHCnL72vedxjQkDDP1mXWo6uco",
                RESTAURANT_1.clone(),
            );

            let query_msg = QueryMsg::GetRestaurants {};
            let res: GetRestaurantsResponse = app
                .wrap()
                .query_wasm_smart(contract_addr.clone(), &query_msg)
                .unwrap();

            assert_eq!(res.restaurants.len(), 1);
            assert_eq!(res.restaurants[0].name, "Test Restaurant");
            assert_eq!(
                res.restaurants[0].image_uri,
                "https://ipfs.io/ipfs/QmXoypizjW3WknFiJnKLwHCnL72vedxjQkDDP1mXWo6uco"
            );
        }

        #[test]
        fn test_get_order_cost_and_create_order() {
            let (mut app, contract_addr) = proper_instantiate();

            let _user_addr = Addr::unchecked(USER);

            register_restaurant(
                &mut app,
                &contract_addr,
                USER,
                "Test Restaurant",
                "https://ipfs.io/ipfs/QmXoypizjW3WknFiJnKLwHCnL72vedxjQkDDP1mXWo6uco",
                RESTAURANT_1.clone(),
            );

            let restaurant_id = format!("restaurant_{}", USER);
            println!("restaurant_id: {}", restaurant_id);

            add_menu_item(
                &mut app,
                &contract_addr,
                USER,
                &restaurant_id,
                "item_1",
                "Pizza",
                Uint128::new(100),
                "https://ipfs.io/ipfs/QmXoypizjW3WknFiJnKLwHCnL72vedxjQkDDP1mXWo6uco",
            );

            let query_msg = QueryMsg::GetOrderCost {
                restaurant_id: restaurant_id.clone(),
                items: vec![OrderItem {
                    item_id: "item_1".to_string(),
                    quantity: 2,
                }],
            };
            let cost_res = app
                .wrap()
                .query_wasm_smart(contract_addr.clone(), &query_msg);
            println!("Query result: {:?}", cost_res); // Debug output
            let cost: GetOrderCostResponse = cost_res.unwrap();
            assert_eq!(cost.total, Uint128::new(200));

            let res = create_order(
                &mut app,
                &contract_addr,
                USER,
                &restaurant_id,
                vec![OrderItem {
                    item_id: "item_1".to_string(),
                    quantity: 2,
                }],
                vec![Coin {
                    denom: NATIVE_DENOM.to_string(),
                    amount: Uint128::new(200),
                }],
            );

            let order_id = res
                .unwrap()
                .events
                .iter()
                .flat_map(|event| event.attributes.iter())
                .find(|attr| attr.key == "order_id")
                .expect("order_id attribute not found")
                .value
                .clone();

            // get order details
            let order_query = QueryMsg::GetUserOrders {
                address: Addr::unchecked(USER).to_string(),
            };
            let orders_res: GetOrdersResponse = app
                .wrap()
                .query_wasm_smart(contract_addr.clone(), &order_query)
                .unwrap();
            let order = orders_res
                .orders
                .iter()
                .find(|o| o.id == order_id)
                .expect("Order not found in GetOrders response");
            assert_eq!(order.total, Uint128::new(200));

            // get escrow details
            let escrow_query = QueryMsg::GetEscrow {
                order_id: order_id.clone(),
            };
            let escrow_res: GetEscrowResponse = app
                .wrap()
                .query_wasm_smart(contract_addr.clone(), &escrow_query)
                .unwrap();
            assert_eq!(escrow_res.escrow.amount, Uint128::new(200));
        }

        #[test]
        fn test_get_escrow() {
            let (mut app, contract_addr) = proper_instantiate();

            register_restaurant(
                &mut app,
                &contract_addr,
                USER,
                "Test Restaurant",
                "https://ipfs.io/ipfs/QmXoypizjW3WknFiJnKLwHCnL72vedxjQkDDP1mXWo6uco",
                RESTAURANT_1.clone(),
            );

            let restaurant_id = format!("restaurant_{}", USER);

            add_menu_item(
                &mut app,
                &contract_addr,
                USER,
                &restaurant_id,
                "item_1",
                "Pizza",
                Uint128::new(100),
                "https://ipfs.io/ipfs/QmXoypizjW3WknFiJnKLwHCnL72vedxjQkDDP1mXWo6uco",
            );

            let create_order_msg = ExecuteMsg::CreateOrder {
                restaurant_id: restaurant_id.clone(),
                items: vec![OrderItem {
                    item_id: "item_1".to_string(),
                    quantity: 2,
                }],
            };
            let res = app
                .execute_contract(
                    Addr::unchecked(USER),
                    contract_addr.clone(),
                    &create_order_msg,
                    &[Coin {
                        denom: NATIVE_DENOM.to_string(),
                        amount: Uint128::new(200),
                    }],
                )
                .unwrap();

            // get order_id from response attributes
            let order_id = res
                .events
                .iter()
                .flat_map(|event| event.attributes.iter())
                .find(|attr| attr.key == "order_id")
                .expect("order_id attribute not found")
                .value
                .clone();

            let query_msg = QueryMsg::GetEscrow {
                order_id: order_id.clone(),
            };
            let res: GetEscrowResponse = app
                .wrap()
                .query_wasm_smart(contract_addr.clone(), &query_msg)
                .unwrap();

            assert_eq!(res.escrow.order_id, order_id);
            assert_eq!(res.escrow.amount, Uint128::new(200));
            assert_eq!(res.escrow.released, false);
        }

        #[test]
        fn test_get_menu_items() {
            let (mut app, contract_addr) = proper_instantiate();

            register_restaurant(
                &mut app,
                &contract_addr,
                USER,
                "Test Restaurant",
                "https://ipfs.io/ipfs/QmXoypizjW3WknFiJnKLwHCnL72vedxjQkDDP1mXWo6uco",
                RESTAURANT_1.clone(),
            );

            let restaurant_id = format!("restaurant_{}", USER);

            add_menu_item(
                &mut app,
                &contract_addr,
                USER,
                &restaurant_id,
                "item_1",
                "Pizza",
                Uint128::new(100),
                "https://ipfs.io/ipfs/QmXoypizjW3WknFiJnKLwHCnL72vedxjQkDDP1mXWo6uco",
            );

            let query_msg = QueryMsg::GetMenuItems {
                restaurant_id: restaurant_id.clone(),
            };
            let res: GetMenuItemsResponse = app
                .wrap()
                .query_wasm_smart(contract_addr.clone(), &query_msg)
                .unwrap();

            assert_eq!(res.menu_items.len(), 1);
            assert_eq!(res.menu_items[0].name, "Pizza");
        }

        #[test]
        fn test_get_orders() {
            let (mut app, contract_addr) = proper_instantiate();

            register_restaurant(
                &mut app,
                &contract_addr,
                USER,
                "Test Restaurant",
                "https://ipfs.io/ipfs/QmXoypizjW3WknFiJnKLwHCnL72vedxjQkDDP1mXWo6uco",
                RESTAURANT_1.clone(),
            );

            let restaurant_id = format!("restaurant_{}", USER);

            add_menu_item(
                &mut app,
                &contract_addr,
                USER,
                &restaurant_id,
                "item_1",
                "Pizza",
                Uint128::new(100),
                "https://ipfs.io/ipfs/QmXoypizjW3WknFiJnKLwHCnL72vedxjQkDDP1mXWo6uco",
            );

            let create_order_msg = ExecuteMsg::CreateOrder {
                restaurant_id: restaurant_id.clone(),
                items: vec![OrderItem {
                    item_id: "item_1".to_string(),
                    quantity: 2,
                }],
            };
            app.execute_contract(
                Addr::unchecked(USER),
                contract_addr.clone(),
                &create_order_msg,
                &[Coin {
                    denom: NATIVE_DENOM.to_string(),
                    amount: Uint128::new(200),
                }],
            )
            .unwrap();

            let query_msg = QueryMsg::GetOrdersFromARestaurant {
                restaurant_id: restaurant_id.clone(),
            };
            let res: GetOrdersResponse = app
                .wrap()
                .query_wasm_smart(contract_addr.clone(), &query_msg)
                .unwrap();

            assert_eq!(res.orders.len(), 1);
            assert_eq!(res.orders[0].restaurant_id, restaurant_id);
        }

        #[test]
        fn test_get_order_status_by_id() {
            let (mut app, contract_addr) = proper_instantiate();

            register_restaurant(
                &mut app,
                &contract_addr,
                USER,
                "Test Restaurant",
                "https://ipfs.io/ipfs/QmXoypizjW3WknFiJnKLwHCnL72vedxjQkDDP1mXWo6uco",
                RESTAURANT_1.clone(),
            );

            let restaurant_id = format!("restaurant_{}", USER);

            add_menu_item(
                &mut app,
                &contract_addr,
                USER,
                &restaurant_id,
                "item_1",
                "Pizza",
                Uint128::new(100),
                "https://ipfs.io/ipfs/QmXoypizjW3WknFiJnKLwHCnL72vedxjQkDDP1mXWo6uco",
            );

            let res = create_order(
                &mut app,
                &contract_addr,
                USER,
                &restaurant_id,
                vec![OrderItem {
                    item_id: "item_1".to_string(),
                    quantity: 2,
                }],
                vec![Coin {
                    denom: NATIVE_DENOM.to_string(),
                    amount: Uint128::new(200),
                }],
            );

            let order_id = res
                .unwrap()
                .events
                .iter()
                .flat_map(|event| event.attributes.iter())
                .find(|attr| attr.key == "order_id")
                .expect("order_id attribute not found")
                .value
                .clone();

            let query_msg = QueryMsg::GetOrderStatusById {
                order_id: order_id.clone(),
            };
            let res: GetOrderStatusResponse = app
                .wrap()
                .query_wasm_smart(contract_addr.clone(), &query_msg)
                .unwrap();

            assert_eq!(res.order_id, order_id);
            assert_eq!(res.status, OrderStatus::Created);
        }

        #[test]
        fn test_register_rider() {
            let (mut app, contract_addr) = proper_instantiate();

            let _res = register_rider(&mut app, &contract_addr, USER, "Test Rider".to_string());
            let rider_id = format!("rider_{}", USER);

            let get_rider = QueryMsg::GetRiderById {
                rider_id: rider_id.clone(),
            };

            // get returns GetRiderResponse
            let response: GetRiderResponse = app
                .wrap()
                .query_wasm_smart(contract_addr.clone(), &get_rider)
                .unwrap();

            // Unwrap the Option<Rider> and verify
            let rider = response.rider.expect("Rider not found");
            assert_eq!(rider.name, "Test Rider");
            assert_eq!(rider.wallet, USER);
            assert_eq!(rider.id, rider_id);
        }

        #[test]
        fn test_escrow_release_on_delivery() {
            let (mut app, contract_addr) = proper_instantiate();

            let restaurant_address = Addr::unchecked("xion1restaurant_wallet");
            register_restaurant(
                &mut app,
                &contract_addr,
                USER,
                "Test Restaurant",
                "https://ipfs.io/ipfs/QmXoypizjW3WknFiJnKLwHCnL72vedxjQkDDP1mXWo6uco",
                restaurant_address.clone(),
            );

            let restaurant_id = format!("restaurant_{}", USER);
            add_menu_item(
                &mut app,
                &contract_addr,
                USER,
                &restaurant_id,
                "item_1",
                "Pizza",
                Uint128::new(100),
                "https://ipfs.io/ipfs/QmXoypizjW3WknFiJnKLwHCnL72vedxjQkDDP1mXWo6uco",
            );

            // Fund USER
            app.init_modules(|router, _, storage| {
                router
                    .bank
                    .init_balance(
                        storage,
                        &Addr::unchecked(USER),
                        vec![Coin {
                            denom: "uxion".to_string(),
                            amount: Uint128::new(1000),
                        }],
                    )
                    .unwrap();
            });

            let total_order_value = Uint128::new(200);
            let res = create_order(
                &mut app,
                &contract_addr,
                USER,
                &restaurant_id,
                vec![OrderItem {
                    item_id: "item_1".to_string(),
                    quantity: 2,
                }],
                vec![Coin {
                    denom: "uxion".to_string(),
                    amount: total_order_value,
                }],
            );

            let order_id = res
                .unwrap()
                .events
                .iter()
                .flat_map(|event| event.attributes.iter())
                .find(|attr| attr.key == "order_id")
                .expect("order_id attribute not found")
                .value
                .clone();

            app.execute_contract(
                Addr::unchecked(USER),
                contract_addr.clone(),
                &ExecuteMsg::AcceptOrder {
                    order_id: order_id.clone(),
                },
                &[],
            )
            .unwrap();

            let _ = register_rider(&mut app, &contract_addr, USER, "Test Rider".to_string());
            let rider_id = format!("rider_{}", USER);
            app.execute_contract(
                Addr::unchecked(USER),
                contract_addr.clone(),
                &ExecuteMsg::AssignRider {
                    order_id: order_id.clone(),
                    rider_id: rider_id.clone(),
                },
                &[],
            )
            .unwrap();

            let confirm_delivery_msg = ExecuteMsg::ConfirmDelivery {
                order_id: order_id.clone(),
            };
            app.execute_contract(
                Addr::unchecked(USER),
                contract_addr.clone(),
                &confirm_delivery_msg,
                &[],
            )
            .unwrap();

            let fee_balance = app.wrap().query_balance(&*FEE_WALLET, "uxion").unwrap();
            assert_eq!(fee_balance.amount, Uint128::new(10));

            let restaurant_balance = app
                .wrap()
                .query_balance(&restaurant_address, "uxion")
                .unwrap();
            assert_eq!(restaurant_balance.amount, Uint128::new(190));
        }

        #[test]
        fn test_user_registration() {
            let (mut app, contract_addr) = proper_instantiate();
            let sender = "xion1useraddress";
            let generated_id = format!("user_{}", sender);

            let _res = app
                .execute_contract(
                    Addr::unchecked(sender),
                    Addr::unchecked(contract_addr.clone()),
                    &ExecuteMsg::RegisterUser {
                        name: "Test User".to_string(),
                        phone_number: "1234567890".to_string(),
                    },
                    &[],
                )
                .unwrap();

            let get_user = QueryMsg::GetUser {
                id: generated_id.clone(),
            };
            let response: UserResponse = app
                .wrap()
                .query_wasm_smart(contract_addr.clone(), &get_user)
                .unwrap();

            assert_eq!(response.name, "Test User");
            assert_eq!(response.wallet, sender);
            assert_eq!(response.id, generated_id);
            assert_eq!(response.phone_number, "1234567890");
            assert_eq!(response.is_registered, true);
        }
    }
}
