// #[cfg(test)]
// mod tests {
//     use crate::contract::{execute, instantiate, query};
//     use crate::msg::{
//         ExecuteMsg, GetEscrowResponse, GetRestaurantsResponse, InstantiateMsg, OrderItem, QueryMsg,
//     };

//     use crate::state::OrderStatus;
//     use cosmwasm_std::{Addr, Coin, Decimal, Empty, Uint128};
//     use cw_multi_test::{App, AppBuilder, AppResponse, Contract, ContractWrapper, Executor};
//     use lazy_static::lazy_static;

//     pub fn contract_template() -> Box<dyn Contract<Empty>> {
//         let contract = ContractWrapper::new(execute, instantiate, query);
//         Box::new(contract)
//     }

//     const USER: &str = "USER";
//     const ADMIN: &str = "ADMIN";
//     const NATIVE_DENOM: &str = "uxion";

//     lazy_static! {
//         static ref RESTAURANT_1: Addr = Addr::unchecked("one");
//         static ref RESTAURANT_2: Addr = Addr::unchecked("two");
//     }

//     fn mock_app() -> App {
//         AppBuilder::new().build(|router, _, storage| {
//             router
//                 .bank
//                 .init_balance(
//                     storage,
//                     &Addr::unchecked(USER),
//                     vec![Coin {
//                         denom: NATIVE_DENOM.to_string(),
//                         amount: Uint128::new(100000),
//                     }],
//                 )
//                 .unwrap();
//         })
//     }

//     fn proper_instantiate() -> (App, Addr) {
//         let mut app = mock_app();
//         let contract_id = app.store_code(contract_template());

//         let fee_address = Addr::unchecked("fee_wallet");

//         let msg = InstantiateMsg {
//             platform_name: "Food Delivery Platform".to_string(),
//             platform_description: "A decentralized food delivery platform".to_string(),
//             owner_address: Addr::unchecked(ADMIN),
//             fee_percentage: Decimal::percent(5), // 5% fee
//             fee_address: fee_address.clone(),
//         };

//         let contract_addr = app
//             .instantiate_contract(
//                 contract_id,
//                 Addr::unchecked(ADMIN),
//                 &msg,
//                 &[],
//                 "Restaurant Contract",
//                 None,
//             )
//             .unwrap();

//         (app, contract_addr)
//     }

//     fn register_restaurant(
//         app: &mut App,
//         contract_addr: &Addr,
//         user: &str,
//         name: &str,
//         image_uri: &str,
//         restaurant_address: Addr,
//     ) {
//         let register_msg = ExecuteMsg::RegisterRestaurant {
//             name: name.to_string(),
//             image_uri: image_uri.to_string(),
//             restaurant_address: restaurant_address.clone(),
//         };
//         app.execute_contract(
//             Addr::unchecked(user),
//             contract_addr.clone(),
//             &register_msg,
//             &[],
//         )
//         .unwrap();
//     }

//     fn register_rider(app: &mut App, contract_addr: &Addr, user: &str, name: String) {
//         let register_rider_msg = ExecuteMsg::RegisterRider {
//             name: name.to_string(),
//         };
//         app.execute_contract(
//             Addr::unchecked(user),
//             contract_addr.clone(),
//             &register_rider_msg,
//             &[],
//         )
//         .unwrap();
//     }

//     fn add_menu_item(
//         app: &mut App,
//         contract_addr: &Addr,
//         user: &str,
//         _restaurant_id: &str,
//         item_id: &str,
//         name: &str,
//         price: Uint128,
//         image_uri: &str,
//     ) {
//         let add_menu_item_msg = ExecuteMsg::AddMenuItem {
//             item_id: item_id.to_string(),
//             name: name.to_string(),
//             price,
//             image_uri: image_uri.to_string(),
//         };
//         app.execute_contract(
//             Addr::unchecked(user),
//             contract_addr.clone(),
//             &add_menu_item_msg,
//             &[],
//         )
//         .unwrap();
//     }

//     fn create_order(
//         app: &mut App,
//         contract_addr: &Addr,
//         user: &str,
//         restaurant_id: &str,
//         items: Vec<OrderItem>,
//         funds: Vec<Coin>,
//     ) -> Result<AppResponse, anyhow::Error> {
//         let create_order_msg = ExecuteMsg::CreateOrder {
//             restaurant_id: restaurant_id.to_string(),
//             items,
//         };
//         app.execute_contract(
//             Addr::unchecked(user),
//             contract_addr.clone(),
//             &create_order_msg,
//             &funds,
//         )
//         .map_err(|e| anyhow::anyhow!(e))
//     }

//     mod restaurant_tests {
//         use cosmwasm_std::StdResult;

//         use crate::{
//             msg::{
//                 GetMenuItemsResponse, GetOrderResponse, GetOrderStatusResponse, GetOrdersResponse,
//                 OrderItem,
//             },
//             state::{Escrow, Rider},
//         };

//         use super::*;

//         #[test]
//         fn test_register_and_get_restaurants() {
//             let (mut app, contract_addr) = proper_instantiate();

//             register_restaurant(
//                 &mut app,
//                 &contract_addr,
//                 USER,
//                 "Test Restaurant",
//                 "https://ipfs.io/ipfs/QmXoypizjW3WknFiJnKLwHCnL72vedxjQkDDP1mXWo6uco",
//                 RESTAURANT_1.clone(),
//             );

//             let query_msg = QueryMsg::GetRestaurants {};
//             let res: GetRestaurantsResponse = app
//                 .wrap()
//                 .query_wasm_smart(contract_addr.clone(), &query_msg)
//                 .unwrap();

//             assert_eq!(res.restaurants.len(), 1);
//             assert_eq!(res.restaurants[0].name, "Test Restaurant");
//             assert_eq!(
//                 res.restaurants[0].image_uri,
//                 "https://ipfs.io/ipfs/QmXoypizjW3WknFiJnKLwHCnL72vedxjQkDDP1mXWo6uco"
//             );
//         }

//         #[test]
//         fn test_create_and_get_order() {
//             let (mut app, contract_addr) = proper_instantiate();

//             let restaurant_owner = Addr::unchecked("restaurant_owner");
//             let restaurant_id = format!("restaurant_{}", restaurant_owner);

//             app.init_modules(|router, _, storage| {
//                 router
//                     .bank
//                     .init_balance(
//                         storage,
//                         &restaurant_owner,
//                         vec![Coin {
//                             denom: "uxion".to_string(),
//                             amount: Uint128::new(1000),
//                         }],
//                     )
//                     .unwrap();
//                 router
//                     .bank
//                     .init_balance(
//                         storage,
//                         &Addr::unchecked(USER),
//                         vec![Coin {
//                             denom: "uxion".to_string(),
//                             amount: Uint128::new(1000),
//                         }],
//                     )
//                     .unwrap();
//             });

//             let register_res = app
//                 .execute_contract(
//                     restaurant_owner.clone(),
//                     contract_addr.clone(),
//                     &ExecuteMsg::RegisterRestaurant {
//                         name: "Test Restaurant".to_string(),
//                         image_uri: "https://ipfs.io/ipfs/...".to_string(),
//                         restaurant_address: restaurant_owner.clone(),
//                     },
//                     &[],
//                 )
//                 .unwrap();
//             println!("Registered Restaurants: {:?}", register_res);

//             let add_item_res = app
//                 .execute_contract(
//                     restaurant_owner.clone(),
//                     contract_addr.clone(),
//                     &ExecuteMsg::AddMenuItem {
//                         item_id: "item_1".to_string(),
//                         name: "Pizza".to_string(),
//                         price: Uint128::new(100),
//                         image_uri: "https://ipfs.io/ipfs/...".to_string(),
//                     },
//                     &[],
//                 )
//                 .unwrap();
//             println!("AddMenuItem result: {:?}", add_item_res);

//             let create_res = app.execute_contract(
//                 Addr::unchecked(USER),
//                 contract_addr.clone(),
//                 &ExecuteMsg::CreateOrder {
//                     restaurant_id: restaurant_id.clone(),
//                     items: vec![OrderItem {
//                         item_id: "item_1".to_string(),
//                         quantity: 2,
//                     }],
//                 },
//                 &[Coin {
//                     denom: "uxion".to_string(),
//                     amount: Uint128::new(200),
//                 }],
//             );
//             assert!(
//                 create_res.is_ok(),
//                 "CreateOrder failed: {:?}",
//                 create_res.err()
//             );
//             println!("CreateOrder result: {:?}", create_res);

//             let raw_response: StdResult<cosmwasm_std::Binary> = app.wrap().query_wasm_smart(
//                 contract_addr.clone(),
//                 &QueryMsg::GetOrder {
//                     id: "order_1".to_string(),
//                 },
//             );
//             println!("Raw GetOrder response: {:?}", raw_response);

//             if let Ok(binary) = raw_response {
//                 let json_str = String::from_utf8(binary.to_vec())
//                     .unwrap_or_else(|_| "Invalid UTF-8".to_string());
//                 println!("Raw JSON: {}", json_str);
//                 let order_res: StdResult<GetOrderResponse> = cosmwasm_std::from_json(&binary);
//                 match order_res {
//                     Ok(get_order_response) => {
//                         println!("Deserialized GetOrderResponse: {:?}", get_order_response);
//                         let order = get_order_response.order;
//                         assert_eq!(order.total, Uint128::new(200));
//                     }
//                     Err(e) => println!("Deserialization failed: {:?}", e),
//                 }
//             } else {
//                 println!("Query failed: {:?}", raw_response.err());
//             }

//             let escrow_res: StdResult<Escrow> = app.wrap().query_wasm_smart(
//                 contract_addr.clone(),
//                 &QueryMsg::GetEscrow {
//                     order_id: "order_1".to_string(),
//                 },
//             );
//             match escrow_res {
//                 Ok(escrow) => {
//                     println!("Escrow: {:?}", escrow);
//                     assert_eq!(escrow.amount, Uint128::new(200));
//                     assert_eq!(escrow.released, false);
//                 }
//                 Err(e) => println!("Escrow query failed: {:?}", e),
//             }
//         }

//         #[test]
//         fn test_get_escrow() {
//             let (mut app, contract_addr) = proper_instantiate();

//             register_restaurant(
//                 &mut app,
//                 &contract_addr,
//                 USER,
//                 "Test Restaurant",
//                 "https://ipfs.io/ipfs/QmXoypizjW3WknFiJnKLwHCnL72vedxjQkDDP1mXWo6uco",
//                 RESTAURANT_1.clone(),
//             );

//             let restaurant_id = format!("restaurant_{}", USER);

//             add_menu_item(
//                 &mut app,
//                 &contract_addr,
//                 USER,
//                 &restaurant_id,
//                 "item_1",
//                 "Pizza",
//                 Uint128::new(100),
//                 "https://ipfs.io/ipfs/QmXoypizjW3WknFiJnKLwHCnL72vedxjQkDDP1mXWo6uco",
//             );

//             let create_order_msg = ExecuteMsg::CreateOrder {
//                 restaurant_id: restaurant_id.clone(),
//                 items: vec![OrderItem {
//                     item_id: "item_1".to_string(),
//                     quantity: 2,
//                 }],
//             };
//             app.execute_contract(
//                 Addr::unchecked(USER),
//                 contract_addr.clone(),
//                 &create_order_msg,
//                 &[Coin {
//                     denom: NATIVE_DENOM.to_string(),
//                     amount: Uint128::new(200),
//                 }],
//             )
//             .unwrap();

//             let query_msg = QueryMsg::GetEscrow {
//                 order_id: "order_1".to_string(),
//             };
//             let res: GetEscrowResponse = app
//                 .wrap()
//                 .query_wasm_smart(contract_addr.clone(), &query_msg)
//                 .unwrap();

//             assert_eq!(res.escrow.order_id, "order_1");
//             assert_eq!(res.escrow.amount, Uint128::new(200));
//             assert_eq!(res.escrow.released, false);
//         }

//         #[test]
//         fn test_get_menu_items() {
//             let (mut app, contract_addr) = proper_instantiate();

//             register_restaurant(
//                 &mut app,
//                 &contract_addr,
//                 USER,
//                 "Test Restaurant",
//                 "https://ipfs.io/ipfs/QmXoypizjW3WknFiJnKLwHCnL72vedxjQkDDP1mXWo6uco",
//                 RESTAURANT_1.clone(),
//             );

//             let restaurant_id = format!("restaurant_{}", USER);

//             add_menu_item(
//                 &mut app,
//                 &contract_addr,
//                 USER,
//                 &restaurant_id,
//                 "item_1",
//                 "Pizza",
//                 Uint128::new(100),
//                 "https://ipfs.io/ipfs/QmXoypizjW3WknFiJnKLwHCnL72vedxjQkDDP1mXWo6uco",
//             );

//             let query_msg = QueryMsg::GetMenuItems {
//                 restaurant_id: restaurant_id.clone(),
//             };
//             let res: GetMenuItemsResponse = app
//                 .wrap()
//                 .query_wasm_smart(contract_addr.clone(), &query_msg)
//                 .unwrap();

//             assert_eq!(res.menu_items.len(), 1);
//             assert_eq!(res.menu_items[0].name, "Pizza");
//         }

//         #[test]
//         fn test_get_orders() {
//             let (mut app, contract_addr) = proper_instantiate();

//             register_restaurant(
//                 &mut app,
//                 &contract_addr,
//                 USER,
//                 "Test Restaurant",
//                 "https://ipfs.io/ipfs/QmXoypizjW3WknFiJnKLwHCnL72vedxjQkDDP1mXWo6uco",
//                 RESTAURANT_1.clone(),
//             );

//             let restaurant_id = format!("restaurant_{}", USER);

//             add_menu_item(
//                 &mut app,
//                 &contract_addr,
//                 USER,
//                 &restaurant_id,
//                 "item_1",
//                 "Pizza",
//                 Uint128::new(100),
//                 "https://ipfs.io/ipfs/QmXoypizjW3WknFiJnKLwHCnL72vedxjQkDDP1mXWo6uco",
//             );

//             let create_order_msg = ExecuteMsg::CreateOrder {
//                 restaurant_id: restaurant_id.clone(),
//                 items: vec![OrderItem {
//                     item_id: "item_1".to_string(),
//                     quantity: 2,
//                 }],
//             };
//             app.execute_contract(
//                 Addr::unchecked(USER),
//                 contract_addr.clone(),
//                 &create_order_msg,
//                 &[Coin {
//                     denom: NATIVE_DENOM.to_string(),
//                     amount: Uint128::new(200),
//                 }],
//             )
//             .unwrap();

//             let query_msg = QueryMsg::GetOrdersFromARestaurant {
//                 restaurant_id: restaurant_id.clone(),
//             };
//             let res: GetOrdersResponse = app
//                 .wrap()
//                 .query_wasm_smart(contract_addr.clone(), &query_msg)
//                 .unwrap();

//             assert_eq!(res.orders.len(), 1);
//             assert_eq!(res.orders[0].restaurant_id, restaurant_id);
//         }

//         #[test]
//         fn test_get_order_status_by_id() {
//             let (mut app, contract_addr) = proper_instantiate();

//             register_restaurant(
//                 &mut app,
//                 &contract_addr,
//                 USER,
//                 "Test Restaurant",
//                 "https://ipfs.io/ipfs/QmXoypizjW3WknFiJnKLwHCnL72vedxjQkDDP1mXWo6uco",
//                 RESTAURANT_1.clone(),
//             );

//             let restaurant_id = format!("restaurant_{}", USER);

//             add_menu_item(
//                 &mut app,
//                 &contract_addr,
//                 USER,
//                 &restaurant_id,
//                 "item_1",
//                 "Pizza",
//                 Uint128::new(100),
//                 "https://ipfs.io/ipfs/QmXoypizjW3WknFiJnKLwHCnL72vedxjQkDDP1mXWo6uco",
//             );

//             let _ = create_order(
//                 &mut app,
//                 &contract_addr,
//                 USER,
//                 &restaurant_id,
//                 vec![OrderItem {
//                     item_id: "item_1".to_string(),
//                     quantity: 2,
//                 }],
//                 vec![Coin {
//                     denom: NATIVE_DENOM.to_string(),
//                     amount: Uint128::new(200),
//                 }],
//             );

//             let query_msg = QueryMsg::GetOrderStatusById {
//                 order_id: "order_1".to_string(),
//             };
//             let res: GetOrderStatusResponse = app
//                 .wrap()
//                 .query_wasm_smart(contract_addr.clone(), &query_msg)
//                 .unwrap();

//             assert_eq!(res.order_id, "order_1");
//             assert_eq!(res.status, OrderStatus::Created);
//         }

//         #[test]
//         fn test_register_rider() {
//             let (mut app, contract_addr) = proper_instantiate();

//             let _res = register_rider(&mut app, &contract_addr, USER, "Test Rider".to_string());

//             let rider_id = format!("rider_{}", USER);

//             let get_rider = QueryMsg::GetRider {
//                 rider_id: rider_id.clone(),
//             };

//             let rider: Rider = app
//                 .wrap()
//                 .query_wasm_smart(contract_addr.clone(), &get_rider)
//                 .unwrap();

//             assert_eq!(rider.name, "Test Rider");
//             assert_eq!(rider.wallet, USER);
//         }

//         #[test]
//         fn test_escrow_release_on_delivery() {
//             let (mut app, contract_addr) = proper_instantiate();

//             let restaurant_address = Addr::unchecked("restaurant_wallet");
//             register_restaurant(
//                 &mut app,
//                 &contract_addr,
//                 USER,
//                 "Test Restaurant",
//                 "https://ipfs.io/ipfs/QmXoypizjW3WknFiJnKLwHCnL72vedxjQkDDP1mXWo6uco",
//                 restaurant_address.clone(),
//             );

//             let restaurant_id = format!("restaurant_{}", USER);
//             add_menu_item(
//                 &mut app,
//                 &contract_addr,
//                 USER,
//                 &restaurant_id,
//                 "item_1",
//                 "Pizza",
//                 Uint128::new(100),
//                 "https://ipfs.io/ipfs/QmXoypizjW3WknFiJnKLwHCnL72vedxjQkDDP1mXWo6uco",
//             );

//             // Fund USER
//             app.init_modules(|router, _, storage| {
//                 router
//                     .bank
//                     .init_balance(
//                         storage,
//                         &Addr::unchecked(USER),
//                         vec![Coin {
//                             denom: "uxion".to_string(),
//                             amount: Uint128::new(1000),
//                         }],
//                     )
//                     .unwrap();
//             });

//             let total_order_value = Uint128::new(200);
//             let _ = create_order(
//                 &mut app,
//                 &contract_addr,
//                 USER,
//                 &restaurant_id,
//                 vec![OrderItem {
//                     item_id: "item_1".to_string(),
//                     quantity: 2,
//                 }],
//                 vec![Coin {
//                     denom: "uxion".to_string(),
//                     amount: total_order_value,
//                 }],
//             );

//             app.execute_contract(
//                 Addr::unchecked(USER),
//                 contract_addr.clone(),
//                 &ExecuteMsg::AcceptOrder {
//                     order_id: "order_1".to_string(),
//                 },
//                 &[],
//             )
//             .unwrap();

//             let _ = register_rider(&mut app, &contract_addr, USER, "Test Rider".to_string());
//             let rider_id = format!("rider_{}", USER);
//             app.execute_contract(
//                 Addr::unchecked(USER),
//                 contract_addr.clone(),
//                 &ExecuteMsg::AssignRider {
//                     order_id: "order_1".to_string(),
//                     rider_id: rider_id.clone(),
//                 },
//                 &[],
//             )
//             .unwrap();

//             let confirm_delivery_msg = ExecuteMsg::ConfirmDelivery {
//                 order_id: "order_1".to_string(),
//             };
//             app.execute_contract(
//                 Addr::unchecked(USER),
//                 contract_addr.clone(),
//                 &confirm_delivery_msg,
//                 &[],
//             )
//             .unwrap();

//             let fee_address = Addr::unchecked("fee_wallet");
//             let fee_balance = app.wrap().query_balance(&fee_address, "uxion").unwrap();
//             assert_eq!(fee_balance.amount, Uint128::new(10));

//             let restaurant_balance = app
//                 .wrap()
//                 .query_balance(&restaurant_address, "uxion")
//                 .unwrap();
//             assert_eq!(restaurant_balance.amount, Uint128::new(190));
//         }
//     }
// }
