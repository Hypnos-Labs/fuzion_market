#![cfg(test)]

pub use self::create_contract::*;
pub use self::create_users::*;
pub use self::init_contracts::init_all_contracts;
use crate::integration_tests_imports::*;
pub use cw_multi_test::{App, Contract, ContractWrapper, Executor};

const VALID_NATIVE: &str = "ujunox";

pub fn here(ctx: impl Display, line: impl Display, col: impl Display) -> String {
    format!(
        "~~~~~~~~~~~~~~~~~~~ \n \n {} \n line {} | column {} \n ________________________",
        ctx, line, col
    )
}

pub mod create_contract {
    use crate::integration_tests::{Contract, ContractWrapper, Empty};

    pub fn cw20_contract() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            cw20_base::contract::execute,
            cw20_base::contract::instantiate,
            cw20_base::contract::query,
        );
        Box::new(contract)
    }

    pub fn cw721_contract() -> Box<dyn Contract<Empty>> {
        let nft_contract = ContractWrapper::new(
            cw721_base::entry::execute,
            cw721_base::entry::instantiate,
            cw721_base::entry::query,
        );
        Box::new(nft_contract)
    }

    pub fn fuzionmarket_contract() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        );

        Box::new(contract)
    }
}

pub mod create_users {
    use super::VALID_NATIVE;
    use cosmwasm_std::Addr;
    use cw_multi_test::App;
    use std::borrow::BorrowMut;

    pub struct User {
        pub name: String,
        pub address: Addr,
    }

    pub fn fake_user(name: String) -> User {
        User {
            name: name.clone(),
            address: Addr::unchecked(name),
        }
    }

    pub fn give_natives<'a>(user: &User, router: &'a mut App) -> &'a mut App {
        let valid_native = cosmwasm_std::coin(100_000_000, VALID_NATIVE);

        router.borrow_mut().init_modules(|router, _, storage| {
            router.bank.init_balance(storage, &user.address, vec![valid_native]).unwrap()
        });

        router
    }
}

pub mod init_contracts {
    use super::*;
    use cw721_base::helpers::Cw721Contract;
    use std::marker::PhantomData;

    pub fn init_cw20_contract(
        router: &mut App,
        admin: &Addr,
        token_name: String,
        token_symbol: String,
        initial_user_one: &Addr,
        initial_user_two: &Addr,
        initial_user_three: &Addr,
        //balance: Uint128,
    ) -> Cw20Contract {
        let cw20_id = router.store_code(cw20_contract());
        let msg = cw20_base::msg::InstantiateMsg {
            name: token_name.clone(),
            symbol: token_symbol.clone(),
            decimals: 6,
            initial_balances: vec![
                Cw20Coin {
                    address: initial_user_one.to_string(),
                    amount: Uint128::from(100u32),
                },
                Cw20Coin {
                    address: initial_user_two.to_string(),
                    amount: Uint128::from(100u32),
                },
                Cw20Coin {
                    address: initial_user_three.to_string(),
                    amount: Uint128::from(100u32),
                },
            ],
            mint: None,
            marketing: None,
        };
        let addr = router
            .instantiate_contract(cw20_id, admin.clone(), &msg, &[], "Token Contract", None)
            .unwrap();

        println!("CW20 | Name: {:?} | Symbol: {:?} | Addr: {:?}", token_name, token_symbol, addr);
        Cw20Contract(addr)
    }

    pub fn init_cw721_contract(
        router: &mut App,
        admin_minter: &Addr,
        nft_name: String,
        nft_symbol: String,
    ) -> Cw721Contract<Empty, Empty> {
        let cw721_id = router.store_code(cw721_contract());

        let msg = cw721_base::msg::InstantiateMsg {
            name: nft_name.clone(),
            symbol: nft_symbol.clone(),
            minter: admin_minter.to_string(),
        };

        let addr = router
            .instantiate_contract(cw721_id, admin_minter.clone(), &msg, &[], "NFT Contract", None)
            .unwrap();

        println!("CW721 | Name: {:?} | Symbol: {:?} | Addr: {:?}", nft_name, nft_symbol, addr);
        Cw721Contract(addr, PhantomData, PhantomData)
    }

    pub fn init_jv_contract(router: &mut App, admin: &Addr) -> Addr {
        let jv_id = router.store_code(fuzionmarket_contract());
        let msg = InstantiateMsg {};

        let addr =
            router.instantiate_contract(jv_id, admin.clone(), &msg, &[], "jv", None).unwrap();

        println!("Fuzionmarket | Addr: {:?}", addr);

        addr
    }

    pub fn init_all_contracts(
        router: &mut App,
        contract_admin: &User,
        john: &User,
        sam: &User,
        max: &User,
    ) -> Result<
        (
            Cw20Contract,
            Cw20Contract,
            Cw20Contract,
            Cw721Contract<Empty, Empty>,
            Cw721Contract<Empty, Empty>,
            Addr,
        ),
        anyhow::Error,
    > {
        //~~~~~~~~~~~~~~~~~~~~~~
        // Init CW20 Contracts
        // John, Sam, Max given 100 tokens of each
        let jvone = init_contracts::init_cw20_contract(
            router,
            &contract_admin.address,
            "jvone".to_string(),
            "JVONE".to_string(),
            &john.address,
            &sam.address,
            &max.address,
        );

        let jvtwo = init_contracts::init_cw20_contract(
            router,
            &contract_admin.address,
            "jvtwo".to_string(),
            "JVTWO".to_string(),
            &john.address,
            &sam.address,
            &max.address,
        );

        let jvtre = init_contracts::init_cw20_contract(
            router,
            &contract_admin.address,
            "jvtre".to_string(),
            "JVTRE".to_string(),
            &john.address,
            &sam.address,
            &max.address,
        );

        //~~~~~~~~~~~~~~~~~~~~~
        // Init NFT Contracts
        let neonpeepz = init_contracts::init_cw721_contract(
            router,
            &contract_admin.address,
            "Neon Peepz".to_string(),
            "NEONPEEPZ".to_string(),
        );

        let shittykittyz = init_contracts::init_cw721_contract(
            router,
            &contract_admin.address,
            "Shitty Kittyz".to_string(),
            "SHITKIT".to_string(),
        );

        //~~~~~~~~~~~~~~~~~~~~~
        // Init Fuzionmarket Contract
        let fuzionmarket = init_contracts::init_jv_contract(router, &contract_admin.address);

        //~~~~~~~~~~~~~~~~~~~~~
        // Give 2 NFTs of each collection to John, Sam, Max
        // John: 1, 2
        // Sam: 3, 4
        // Max: 5, 6

        let np_mint_msgs = [
            cw721_base::ExecuteMsg::<Option<Empty>, Empty>::Mint(cw721_base::MintMsg {
                token_id: "1".to_string(),
                owner: john.address.clone().to_string(),
                token_uri: Some("given_to_john_1_neonpeepz".to_string()),
                extension: None, //extension: None
            }),
            cw721_base::ExecuteMsg::<Option<Empty>, Empty>::Mint(cw721_base::MintMsg {
                token_id: "2".to_string(),
                owner: john.address.clone().to_string(),
                token_uri: Some("given_to_john_2_neonpeepz".to_string()),
                extension: None,
            }),
            cw721_base::ExecuteMsg::<Option<Empty>, Empty>::Mint(cw721_base::MintMsg {
                token_id: "3".to_string(),
                owner: sam.address.clone().to_string(),
                token_uri: Some("given_to_sam_3_neonpeepz".to_string()),
                extension: None,
            }),
            cw721_base::ExecuteMsg::<Option<Empty>, Empty>::Mint(cw721_base::MintMsg {
                token_id: "4".to_string(),
                owner: sam.address.clone().to_string(),
                token_uri: Some("given_to_sam_4_neonpeepz".to_string()),
                extension: None,
            }),
            cw721_base::ExecuteMsg::<Option<Empty>, Empty>::Mint(cw721_base::MintMsg {
                token_id: "5".to_string(),
                owner: max.address.clone().to_string(),
                token_uri: Some("given_to_max_5_neonpeepz".to_string()),
                extension: None,
            }),
            cw721_base::ExecuteMsg::<Option<Empty>, Empty>::Mint(cw721_base::MintMsg {
                token_id: "6".to_string(),
                owner: max.address.clone().to_string(),
                token_uri: Some("given_to_max_6_neonpeepz".to_string()),
                extension: None,
            }),
        ];

        let sk_mint_msgs = [
            cw721_base::ExecuteMsg::<Option<Empty>, Empty>::Mint(cw721_base::MintMsg {
                token_id: "1".to_string(),
                owner: john.address.clone().to_string(),
                token_uri: Some("given_to_john_1_shittykittyz".to_string()),
                extension: None,
            }),
            cw721_base::ExecuteMsg::<Option<Empty>, Empty>::Mint(cw721_base::MintMsg {
                token_id: "2".to_string(),
                owner: john.address.clone().to_string(),
                token_uri: Some("given_to_john_2_shittykittyz".to_string()),
                extension: None,
            }),
            cw721_base::ExecuteMsg::<Option<Empty>, Empty>::Mint(cw721_base::MintMsg {
                token_id: "3".to_string(),
                owner: sam.address.clone().to_string(),
                token_uri: Some("given_to_sam_3_shittykittyz".to_string()),
                extension: None,
            }),
            cw721_base::ExecuteMsg::<Option<Empty>, Empty>::Mint(cw721_base::MintMsg {
                token_id: "4".to_string(),
                owner: sam.address.clone().to_string(),
                token_uri: Some("given_to_sam_4_shittykittyz".to_string()),
                extension: None,
            }),
            cw721_base::ExecuteMsg::<Option<Empty>, Empty>::Mint(cw721_base::MintMsg {
                token_id: "5".to_string(),
                owner: max.address.clone().to_string(),
                token_uri: Some("given_to_max_5_shittykittyz".to_string()),
                extension: None,
            }),
            cw721_base::ExecuteMsg::<Option<Empty>, Empty>::Mint(cw721_base::MintMsg {
                token_id: "6".to_string(),
                owner: max.address.clone().to_string(),
                token_uri: Some("given_to_max_6_shittykittyz".to_string()),
                extension: None,
            }),
        ];

        for mint_msg in np_mint_msgs {
            router.execute_contract(
                contract_admin.address.clone(),
                neonpeepz.addr(),
                &mint_msg,
                &[],
            )?;
        }

        for mint_msgx in sk_mint_msgs {
            router.execute_contract(
                contract_admin.address.clone(),
                shittykittyz.addr(),
                &mint_msgx,
                &[],
            )?;
        }

        Ok((jvone, jvtwo, jvtre, neonpeepz, shittykittyz, fuzionmarket))
    }
}

pub mod create_valid_listing {

    use crate::msg::ExecuteMsg;
    use cosmwasm_std::{coin, Addr, Uint128}; //coins
    use cw20::Cw20CoinVerified; //, Cw20Coin};

    use crate::{msg::CreateListingMsg, state::GenericBalance, state::Nft};

    use super::VALID_NATIVE; // REAL_JVONE, REAL_NEONPEEPZ};

    pub fn valid_ask_all(
        jvone_addr: Addr,
        _jvtwo_addr: Addr,
        _jvtre_addr: Addr,
        np_addr: Addr,
        _sk_addr: Addr,
    ) -> ExecuteMsg {
        let valid_native = coin(10, VALID_NATIVE);

        let valid_cw20 = Cw20CoinVerified {
            address: jvone_addr,
            amount: Uint128::from(10u32),
        };

        let valid_nft = Nft {
            contract_address: np_addr,
            token_id: "2".to_string(),
        };

        let valid_ask_price = GenericBalance {
            native: vec![valid_native],
            cw20: vec![valid_cw20],
            nfts: vec![valid_nft],
        };

        let cm = CreateListingMsg {
            //id: 11,
            ask: valid_ask_price,
            //whitelisted_purchasers: None,
            whitelisted_buyer: None,
        };

        crate::msg::ExecuteMsg::CreateListing {
            create_msg: cm,
        }
    }

    pub fn create_valid_ask(
        //listing_id: u64,
        juno_amt: Option<u128>,

        jvone_addr: Option<Addr>,
        jvone_amt: Option<Uint128>,
        jvtwo_addr: Option<Addr>,
        jvtwo_amt: Option<Uint128>,
        jvtre_addr: Option<Addr>,
        jvtre_amt: Option<Uint128>,

        np_addr: Option<Addr>,
        np_id: Option<String>,

        sk_addr: Option<Addr>,
        sk_id: Option<String>,

        //whitelisted_purchasers: Option<Vec<String>>,
        whitelisted_buyer: Option<String>,
    ) -> ExecuteMsg {
        let native_ask = match juno_amt {
            None => vec![],
            Some(a) => vec![coin(a, VALID_NATIVE)],
        };

        let mut cw20_ask: Vec<Cw20CoinVerified> = vec![];

        if let Some(jvoneaddr) = jvone_addr {
            cw20_ask.push(Cw20CoinVerified {
                address: jvoneaddr,
                amount: jvone_amt.unwrap(),
            })
        };

        if let Some(jvtwoaddr) = jvtwo_addr {
            cw20_ask.push(Cw20CoinVerified {
                address: jvtwoaddr,
                amount: jvtwo_amt.unwrap(),
            })
        };

        if let Some(jvtreaddr) = jvtre_addr {
            cw20_ask.push(Cw20CoinVerified {
                address: jvtreaddr,
                amount: jvtre_amt.unwrap(),
            })
        };

        let mut nft_ask: Vec<Nft> = vec![];

        if let Some(npaddr) = np_addr {
            nft_ask.push(Nft {
                contract_address: npaddr,
                token_id: np_id.unwrap(),
            })
        };

        if let Some(skaddr) = sk_addr {
            nft_ask.push(Nft {
                contract_address: skaddr,
                token_id: sk_id.unwrap(),
            })
        };

        let valid_ask_price = GenericBalance {
            native: native_ask,
            cw20: cw20_ask,
            nfts: nft_ask,
        };

        let cm = CreateListingMsg {
            //id: listing_id,
            ask: valid_ask_price,
            //whitelisted_purchasers: whitelisted_purchasers,
            whitelisted_buyer,
        };

        crate::msg::ExecuteMsg::CreateListing {
            create_msg: cm,
        }
    }

    pub fn create_listing_msg(
        //listing_id: u64,
        jvone_addr: Addr,
        np_addr: Addr,
        //whitelist: Option<Vec<Addr>>,
        whitelisted_buyer: Option<String>,
    ) -> CreateListingMsg {
        let native_ask = cosmwasm_std::coins(1, "ujunox");

        let cw20_ask = vec![Cw20CoinVerified {
            address: jvone_addr,
            amount: Uint128::from(1u32),
        }];

        let nft_ask = vec![Nft {
            contract_address: np_addr,
            token_id: "5".to_string(),
        }];

        let ask_price = GenericBalance {
            native: native_ask,
            cw20: cw20_ask,
            nfts: nft_ask,
        };

        // let whitelist =
        //     whitelist.map(|wl| wl.iter().map(|addr| addr.to_string()).collect::<Vec<String>>());

        CreateListingMsg {
            //id: listing_id,
            ask: ask_price,
            //whitelisted_purchasers: whitelist,
            whitelisted_buyer,
        }
    }
}

//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// Listings
//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

// #[test]
// fn generic_balance_compare_test() -> Result<(), anyhow::Error> {
//     //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
//     // Testing of custom compare implementation for GenericBalance
//     //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
//     use anyhow::Result;
//     use cw_multi_test::AppResponse;
//     // Setup
//     let mut router = App::default();
//     let contract_admin = create_users::fake_user("admin".to_string());
//     let john = create_users::fake_user("john".to_string());
//     let sam = create_users::fake_user("sam".to_string());
//     let max = create_users::fake_user("max".to_string());
//     // Instantiate all contracts
//     let (jvone, jvtwo, jvtre, neonpeepz, shittykittyz, fuzionmarket) =
//         init_all_contracts(&mut router, &contract_admin, &john, &sam, &max)?;
//     // Give native balances to all users
//     // Each user gets 100 VALID_NATIVE
//     let router = give_natives(&john, &mut router);
//     let router = give_natives(&sam, router);
//     let router = give_natives(&max, router);
//     Ok(())
// }

#[test]
fn create_listing_should_fail() -> Result<(), anyhow::Error> {
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // Assert Failure on all these create listings
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    //use std::borrow::BorrowMut;
    use anyhow::Result;
    use cw_multi_test::AppResponse;
    // Setup
    let mut router = App::default();
    let contract_admin = create_users::fake_user("admin".to_string());
    let john = create_users::fake_user("john".to_string());
    let sam = create_users::fake_user("sam".to_string());
    let max = create_users::fake_user("max".to_string());

    // Instantiate all contracts
    let (jvone, jvtwo, jvtre, neonpeepz, shittykittyz, fuzionmarket) =
        init_all_contracts(&mut router, &contract_admin, &john, &sam, &max)?;

    // Give native balances to all users
    // Each user gets 100 VALID_NATIVE
    let router = give_natives(&john, &mut router);
    let router = give_natives(&sam, router);
    let router = give_natives(&max, router);
    //let router = give_natives(&bad_actor, &mut router);

    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // Can't create with same ID
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

    let ask_price_valid = create_valid_listing::create_valid_ask(
        //11,
        Some(10),
        Some(jvone.addr()),
        Some(Uint128::from(25u32)),
        Some(jvtwo.addr()),
        Some(Uint128::from(10u32)),
        Some(jvtre.addr()),
        Some(Uint128::from(15u32)),
        Some(neonpeepz.addr()),
        Some("3".to_string()),
        Some(shittykittyz.addr()),
        Some("3".to_string()),
        None,
    );
    let one_juno = coins(1, "ujunox");
    let res: Result<AppResponse> = router.execute_contract(
        john.address.clone(),
        fuzionmarket.clone(),
        &ask_price_valid,
        &one_juno,
    );
    ensure!(res.is_ok(), here("'Testing Ask Creation' failure", line!(), column!()));

    // > REMOVE because Listing ID now comes from incrementor in state <
    // // John can't create another listing with same ID
    // let res: Result<AppResponse> = router.execute_contract(
    //     john.address.clone(),
    //     fuzionmarket.clone(),
    //     &ask_price_valid,
    //     &one_juno,
    // );
    // ensure!(res.is_err(), here("Cant create with same ID failure", line!(), column!()));

    // // Sam can't create another listing with same ID
    // let res: Result<AppResponse> =
    //     router.execute_contract(sam.address.clone(), fuzionmarket, &ask_price_valid, &one_juno);

    // ensure!(res.is_err(), here("Cant create with same ID failure", line!(), column!()));

    Ok(())
}

// <X> Create with Native
// <X> Create with CW20
// <X> Create with NFT
#[test]
fn create_listing_should_pass() -> Result<(), anyhow::Error> {
    use anyhow::Result;
    use cw_multi_test::AppResponse;
    // Setup
    let mut router = App::default();
    let contract_admin = create_users::fake_user("admin".to_string());
    let john = create_users::fake_user("john".to_string());
    let sam = create_users::fake_user("sam".to_string());
    let max = create_users::fake_user("max".to_string());
    let bad_actor = create_users::fake_user("badguy".to_string());

    // Instantiate all contracts
    let (jvone, jvtwo, jvtre, neonpeepz, shittykittyz, fuzionmarket) =
        init_all_contracts(&mut router, &contract_admin, &john, &sam, &max)?;
    // Gives each user
    // 100 JVONE, JVTWO, JVTRE
    // 2 ShittyKittyz + 2 NeonPeepz

    // Give each user 100 VALID_NATIVE
    let router = give_natives(&john, &mut router);
    let router = give_natives(&sam, router);
    let router = give_natives(&max, router);
    let router = give_natives(&bad_actor, router);

    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // Create with a Native
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

    let ask_price_valid = create_valid_listing::create_valid_ask(
        //1,
        Some(10),
        Some(jvone.addr()),
        Some(Uint128::from(25u32)),
        Some(jvtwo.addr()),
        Some(Uint128::from(10u32)),
        Some(jvtre.addr()),
        Some(Uint128::from(15u32)),
        Some(neonpeepz.addr()),
        Some("3".to_string()),
        Some(shittykittyz.addr()),
        Some("3".to_string()),
        None,
    );
    let one_juno = coins(1, "ujunox");
    let res: Result<AppResponse> = router.execute_contract(
        john.address.clone(),
        fuzionmarket.clone(),
        &ask_price_valid,
        &one_juno,
    );
    // passes
    ensure!(res.is_ok(), here("'Testing Ask Creation' failure", line!(), column!()));
    let john_new_balance: Coin =
        router.wrap().query_balance(john.address.to_string(), "ujunox").unwrap();
    ensure!(
        (john_new_balance.amount == Uint128::from(99_999_999_u32)),
        here(format!("John balance: {}", john_new_balance.amount), line!(), column!())
    );

    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // Create with cw20
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

    let cm = create_valid_listing::create_listing_msg(
        //2,
        jvone.addr(),
        neonpeepz.addr(),
        None,
    );
    let cmsg = to_binary(&crate::msg::ReceiveMsg::CreateListingCw20 {
        create_msg: cm,
    })?;
    let createmsg = cw20_base::msg::ExecuteMsg::Send {
        contract: fuzionmarket.to_string(),
        amount: Uint128::from(1u32),
        msg: cmsg,
    };
    let res: Result<AppResponse> =
        router.execute_contract(john.address.clone(), jvone.addr(), &createmsg, &[]);
    ensure!(res.is_ok(), here("'Testing Ask Creation with cw20' failure", line!(), column!()));
    assert_eq!(jvone.balance(&router.wrap(), john.address.clone()), Ok(Uint128::from(99u32)));

    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // Create with NFT
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    let cm = create_valid_listing::create_listing_msg(
        //3,
        jvone.addr(),
        neonpeepz.addr(),
        None,
    );
    let cmsg_nft = to_binary(&crate::msg::ReceiveNftMsg::CreateListingCw721 {
        create_msg: cm,
    })?;
    let createmsg_nft: cw721_base::ExecuteMsg<Option<Empty>, Empty> =
        cw721_base::msg::ExecuteMsg::SendNft {
            contract: fuzionmarket.to_string(),
            token_id: "1".to_string(),
            msg: cmsg_nft,
        };
    let res: Result<AppResponse> =
        router.execute_contract(john.address.clone(), neonpeepz.addr(), &createmsg_nft, &[]);
    ensure!(res.is_ok(), here("'Testing Ask Creation with NFT' failure", line!(), column!()));
    let owner = neonpeepz.owner_of(&router.wrap(), "1".to_string(), false).unwrap().owner;
    assert_eq!(owner, fuzionmarket.to_string());

    Ok(())
}

// Add to Listing
// <X> Can add each type
// <X> Can't add to listing that's not your own
// <X> Balance checks <Native, CW20, NFT>
#[test]
fn add_to_listing() -> Result<(), anyhow::Error> {
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // Adding each asset type
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    use anyhow::Result;
    use cw_multi_test::AppResponse;
    // Setup
    let mut router = App::default();
    let contract_admin = create_users::fake_user("admin".to_string());
    let john = create_users::fake_user("john".to_string());
    let sam = create_users::fake_user("sam".to_string());
    let max = create_users::fake_user("max".to_string());
    let bad_actor = create_users::fake_user("badguy".to_string());

    // Instantiate all contracts
    let (jvone, _jvtwo, _jvtre, _neonpeepz, shittykittyz, fuzionmarket) =
        init_all_contracts(&mut router, &contract_admin, &john, &sam, &max)?;
    // Gives each user
    // 100 JVONE, JVTWO, JVTRE
    // 2 ShittyKittyz + 2 NeonPeepz

    // Give each user 100 VALID_NATIVE
    let router = give_natives(&john, &mut router);
    let router = give_natives(&sam, router);
    let router = give_natives(&max, router);
    let router = give_natives(&bad_actor, router);

    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // Create a basic listing with a Native token
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

    let ask_price_valid = create_valid_listing::create_valid_ask(
        //1,
        Some(10),
        Some(jvone.addr()),
        Some(Uint128::from(25u32)),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    );
    let one_juno = coins(1, "ujunox");
    let res: Result<AppResponse> = router.execute_contract(
        john.address.clone(),
        fuzionmarket.clone(),
        &ask_price_valid,
        &one_juno,
    );
    // passes
    ensure!(res.is_ok(), here("'Testing Ask Creation' failure", line!(), column!()));
    //let john_new_balance = router.wrap().query_all_balances(addr).unwrap();
    let john_new_balance: Coin =
        router.wrap().query_balance(john.address.to_string(), "ujunox").unwrap();
    ensure!(
        (john_new_balance.amount == Uint128::from(99_999_999_u32)),
        here(format!("John balance: {}", john_new_balance.amount), line!(), column!())
    );

    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // Add Native
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

    let add_native_msg = crate::msg::ExecuteMsg::AddToListing {
        listing_id: 1,
    };

    // Sam cannot add
    let res: Result<AppResponse> = router.execute_contract(
        sam.address.clone(),
        fuzionmarket.clone(),
        &add_native_msg,
        &one_juno,
    );
    ensure!(
        res.is_err(),
        here("Sam shouldn't be able to add to John's listing", line!(), column!())
    );
    // ensure Sam's balance has not changed
    let sam_balance: Coin = router.wrap().query_balance(sam.address.to_string(), "ujunox").unwrap();
    ensure!(
        (sam_balance.amount == Uint128::from(100_000_000_u32)),
        here(format!("Sam balance: {}", sam_balance.amount), line!(), column!())
    );

    // John can add
    let res: Result<AppResponse> = router.execute_contract(
        john.address.clone(),
        fuzionmarket.clone(),
        &add_native_msg,
        &one_juno,
    );
    ensure!(res.is_ok(), here("John couldn't add", line!(), column!()));
    // ensure John's balance updated
    let john_newer_balance: Coin =
        router.wrap().query_balance(john.address.to_string(), "ujunox").unwrap();
    ensure!(
        (john_newer_balance.amount == Uint128::from(99_999_998_u32)),
        here(format!("John balance: {}", john_newer_balance.amount), line!(), column!())
    );

    // ensure contract balance updated
    let contract_balance: Coin =
        router.wrap().query_balance(fuzionmarket.to_string(), "ujunox").unwrap();
    ensure!(
        (contract_balance.amount == Uint128::from(2u32)),
        here(format!("Contract balance: {}", contract_balance.amount), line!(), column!())
    );

    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // Add CW20
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

    let add_msg = to_binary(&crate::msg::ReceiveMsg::AddToListingCw20 {
        listing_id: 1,
    })?;

    let add_cw20_msg = cw20_base::msg::ExecuteMsg::Send {
        contract: fuzionmarket.to_string(),
        amount: Uint128::from(1u32),
        msg: add_msg,
    };

    // Sam cannot add jvone to Johns listing
    let res: Result<AppResponse> =
        router.execute_contract(sam.address.clone(), jvone.addr(), &add_cw20_msg, &[]);
    ensure!(
        res.is_err(),
        here("Sam shouldn't be able to add to John's listing", line!(), column!())
    );
    // ensure Sam's balance has not changed
    let q = cw20_base::msg::QueryMsg::Balance {
        address: sam.address.clone().to_string(),
    };
    let sam_jvone_balance: cw20::BalanceResponse =
        router.wrap().query_wasm_smart(jvone.addr(), &q).unwrap();
    ensure!(
        (sam_jvone_balance.balance == Uint128::from(100u32)),
        here(format!("Sam JVONE balance: {}", sam_jvone_balance.balance), line!(), column!())
    );

    // John can add jvone to his own listing
    let res: Result<AppResponse> =
        router.execute_contract(john.address.clone(), jvone.addr(), &add_cw20_msg, &[]);
    ensure!(res.is_ok(), here("John added", line!(), column!()));
    // ensure John's balance updated
    let q = cw20_base::msg::QueryMsg::Balance {
        address: john.address.clone().to_string(),
    };
    let john_jvone_balance: cw20::BalanceResponse =
        router.wrap().query_wasm_smart(jvone.addr(), &q).unwrap();
    ensure!(
        (john_jvone_balance.balance == Uint128::from(99u32)),
        here(format!("John JVONE balance: {}", john_jvone_balance.balance), line!(), column!())
    );

    // ensure contract balance updated
    let q = cw20_base::msg::QueryMsg::Balance {
        address: fuzionmarket.to_string(),
    };
    let contract_jvone_balance: cw20::BalanceResponse =
        router.wrap().query_wasm_smart(jvone.addr(), &q).unwrap();
    ensure!(
        (contract_jvone_balance.balance == Uint128::from(1u32)),
        here(
            format!("Contract JVONE balance: {}", contract_jvone_balance.balance),
            line!(),
            column!()
        )
    );

    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // Add NFT
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

    let add_msg = to_binary(&crate::msg::ReceiveNftMsg::AddToListingCw721 {
        listing_id: 1,
    })?;

    let john_add_nft_msg: cw721_base::ExecuteMsg<Option<Empty>, Empty> =
        cw721_base::msg::ExecuteMsg::SendNft {
            contract: fuzionmarket.to_string(),
            token_id: "1".to_string(),
            msg: add_msg.clone(),
        };

    let sam_add_nft_msg: cw721_base::ExecuteMsg<Option<Empty>, Empty> =
        cw721_base::msg::ExecuteMsg::SendNft {
            contract: fuzionmarket.to_string(),
            token_id: "3".to_string(),
            msg: add_msg,
        };

    // Sam cannot add nft to Johns listing
    let res: Result<AppResponse> =
        router.execute_contract(sam.address.clone(), shittykittyz.addr(), &sam_add_nft_msg, &[]);
    ensure!(
        res.is_err(),
        here("Sam shouldn't be able to add to John's listing", line!(), column!())
    );
    // ensure Sam still has NFT
    let owner = shittykittyz.owner_of(&router.wrap(), "3".to_string(), false).unwrap().owner;
    assert_eq!(owner, sam.address.clone().to_string());

    // John can add
    let res: Result<AppResponse> =
        router.execute_contract(john.address.clone(), shittykittyz.addr(), &john_add_nft_msg, &[]);
    ensure!(res.is_ok(), here("John can add", line!(), column!()));
    // ensure Contract has NFT
    let owner = shittykittyz.owner_of(&router.wrap(), "1".to_string(), false).unwrap().owner;
    assert_eq!(owner, fuzionmarket.to_string());

    Ok(())
}

// Removing a Listing <pre-finalization>
// <X> Can't remove listing that's not your own
// <X> Listing is deleted after Removal
// <X> Balance checks <Native, CW20, NFT>
#[test]
fn remove_a_listing() -> Result<(), anyhow::Error> {
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // Removing a Listing <pre-finalization>
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    use anyhow::Result;
    use cw_multi_test::AppResponse;
    // Setup
    let mut router = App::default();
    let contract_admin = create_users::fake_user("admin".to_string());
    let john = create_users::fake_user("john".to_string());
    let sam = create_users::fake_user("sam".to_string());
    let max = create_users::fake_user("max".to_string());
    let bad_actor = create_users::fake_user("badguy".to_string());

    // Instantiate all contracts
    let (jvone, jvtwo, _jvtre, neonpeepz, shittykittyz, fuzionmarket) =
        init_all_contracts(&mut router, &contract_admin, &john, &sam, &max)?;
    // Gives each user
    // 100 JVONE, JVTWO, JVTRE
    // 2 ShittyKittyz + 2 NeonPeepz

    // Give each user 100 VALID_NATIVE
    let router = give_natives(&john, &mut router);
    let router = give_natives(&sam, router);
    let router = give_natives(&max, router);
    let router = give_natives(&bad_actor, router);

    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // Create 2 Listings with Native tokens
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

    //~~~~~~~~~~
    // Listing one, created by John
    //~~~~~~~~~~
    let ask_price_valid = create_valid_listing::create_valid_ask(
        // Listing ID
        //1,
        // ujunox in ask
        Some(10),
        // jvone in ask
        Some(jvone.addr()),
        Some(Uint128::from(25u32)),
        // jvtwo in ask
        None,
        None,
        // jvtre in ask
        None,
        None,
        // NeonPeepz in ask
        None,
        None,
        // ShittyKittyz in ask
        None,
        None,
        // whitelisted purchasers
        None,
    );
    // For sale 1 ujunox
    let one_juno = coins(1, "ujunox");

    let res: Result<AppResponse> = router.execute_contract(
        john.address.clone(),
        fuzionmarket.clone(),
        &ask_price_valid,
        &one_juno,
    );
    // passes
    ensure!(res.is_ok(), here("'Testing Ask Creation' failure", line!(), column!()));
    let john_new_balance: Coin =
        router.wrap().query_balance(john.address.to_string(), "ujunox").unwrap();
    ensure!(
        (john_new_balance.amount == Uint128::from(99_999_999_u32)),
        here(format!("John balance: {}", john_new_balance.amount), line!(), column!())
    );

    //~~~~~~~~~~
    // Listing two, created by Sam
    //~~~~~~~~~~
    let ask_price_valid = create_valid_listing::create_valid_ask(
        // Listing ID
        //21,
        // ujunox in ask
        Some(10),
        // jvone in ask
        Some(jvone.addr()),
        Some(Uint128::from(25u32)),
        // jvtwo in ask
        None,
        None,
        // jvtre in ask
        None,
        None,
        // NeonPeepz in ask
        None,
        None,
        // ShittyKittyz in ask
        None,
        None,
        // whitelisted purchasers
        None,
    );
    // Listing for sale, 1 ujunox
    let one_juno = coins(1, "ujunox");
    let res: Result<AppResponse> = router.execute_contract(
        sam.address.clone(),
        fuzionmarket.clone(),
        &ask_price_valid,
        &one_juno,
    );
    // passes
    ensure!(res.is_ok(), here("'Testing Ask Creation' failure", line!(), column!()));
    //let john_new_balance = router.wrap().query_all_balances(addr).unwrap();
    let sam_new_balance: Coin =
        router.wrap().query_balance(sam.address.to_string(), "ujunox").unwrap();
    ensure!(
        (sam_new_balance.amount == Uint128::from(99_999_999_u32)),
        here(format!("Sam balance: {}", sam_new_balance.amount), line!(), column!())
    );

    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // Add 10 JVONE, 10 JVTWO to each
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

    let john_msg = to_binary(&crate::msg::ReceiveMsg::AddToListingCw20 {
        listing_id: 1,
    })?;
    let sam_msg = to_binary(&crate::msg::ReceiveMsg::AddToListingCw20 {
        listing_id: 2,
    })?;
    let john_add_ten_msg = cw20_base::msg::ExecuteMsg::Send {
        contract: fuzionmarket.to_string(),
        amount: Uint128::from(10u32),
        msg: john_msg,
    };
    let sam_add_ten_msg = cw20_base::msg::ExecuteMsg::Send {
        contract: fuzionmarket.to_string(),
        amount: Uint128::from(10u32),
        msg: sam_msg,
    };

    // John adding ten jvone
    let res: Result<AppResponse> =
        router.execute_contract(john.address.clone(), jvone.addr(), &john_add_ten_msg, &[]);
    ensure!(res.is_ok(), here("John added", line!(), column!()));
    assert_eq!(jvone.balance(&router.wrap(), john.address.clone()), Ok(Uint128::from(90u32)));
    assert_eq!(jvone.balance(&router.wrap(), fuzionmarket.clone()), Ok(Uint128::from(10u32)));
    // John adding ten jvtwo
    let res: Result<AppResponse> =
        router.execute_contract(john.address.clone(), jvtwo.addr(), &john_add_ten_msg, &[]);
    ensure!(res.is_ok(), here("Sam added", line!(), column!()));
    assert_eq!(jvtwo.balance(&router.wrap(), john.address.clone()), Ok(Uint128::from(90u32)));
    assert_eq!(jvtwo.balance(&router.wrap(), fuzionmarket.clone()), Ok(Uint128::from(10u32)));
    // ~~~
    // Sam adding ten jvone
    let res: Result<AppResponse> =
        router.execute_contract(sam.address.clone(), jvone.addr(), &sam_add_ten_msg, &[]);
    ensure!(res.is_ok(), here("Sam added", line!(), column!()));
    assert_eq!(jvone.balance(&router.wrap(), sam.address.clone()), Ok(Uint128::from(90u32)));
    assert_eq!(jvone.balance(&router.wrap(), fuzionmarket.clone()), Ok(Uint128::from(20u32)));
    // Sam adding ten jvtwo
    let res: Result<AppResponse> =
        router.execute_contract(sam.address.clone(), jvtwo.addr(), &sam_add_ten_msg, &[]);
    ensure!(res.is_ok(), here("sam added", line!(), column!()));
    assert_eq!(jvtwo.balance(&router.wrap(), sam.address.clone()), Ok(Uint128::from(90u32)));
    assert_eq!(jvtwo.balance(&router.wrap(), fuzionmarket.clone()), Ok(Uint128::from(20u32)));

    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // Adding 1 NeonPeep, 1 ShittyKitty to each listing
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

    let john_add_msg = to_binary(&crate::msg::ReceiveNftMsg::AddToListingCw721 {
        listing_id: 1,
    })?;
    let john_add_nft_msg: cw721_base::ExecuteMsg<Option<Empty>, Empty> =
        cw721_base::msg::ExecuteMsg::SendNft {
            contract: fuzionmarket.to_string(),
            token_id: "1".to_string(),
            msg: john_add_msg,
        };

    // John adding NeonPeepz 1 to his listing
    let res: Result<AppResponse> =
        router.execute_contract(john.address.clone(), neonpeepz.addr(), &john_add_nft_msg, &[]);
    ensure!(res.is_ok(), here("john", line!(), column!()));
    // John adding ShittyKittyz 1 to his listing
    let res: Result<AppResponse> =
        router.execute_contract(john.address.clone(), shittykittyz.addr(), &john_add_nft_msg, &[]);
    ensure!(res.is_ok(), here("john", line!(), column!()));
    // Contract has NFTs
    let owner = shittykittyz.owner_of(&router.wrap(), "1".to_string(), false).unwrap().owner;
    let owner2 = neonpeepz.owner_of(&router.wrap(), "1".to_string(), false).unwrap().owner;
    assert_eq!(owner, fuzionmarket.to_string());
    assert_eq!(owner2, fuzionmarket.to_string());

    // // ensure Sam still has NFT
    // let owner = shittykittyz.owner_of(&router.wrap(), "3".to_string(), false).unwrap().owner;
    // assert_eq!(owner, sam.address.clone().to_string());
    let sam_add_msg = to_binary(&crate::msg::ReceiveNftMsg::AddToListingCw721 {
        listing_id: 2,
    })?;
    let sam_add_nft_msg: cw721_base::ExecuteMsg<Option<Empty>, Empty> =
        cw721_base::msg::ExecuteMsg::SendNft {
            contract: fuzionmarket.to_string(),
            token_id: "3".to_string(),
            msg: sam_add_msg,
        };

    // Sam adding NeonPeepz 3 to her listing
    let res: Result<AppResponse> =
        router.execute_contract(sam.address.clone(), neonpeepz.addr(), &sam_add_nft_msg, &[]);
    ensure!(res.is_ok(), here("sam", line!(), column!()));
    // Sam adding ShittyKittyz 3 to her listing
    let res: Result<AppResponse> =
        router.execute_contract(sam.address.clone(), shittykittyz.addr(), &sam_add_nft_msg, &[]);
    ensure!(res.is_ok(), here("sam", line!(), column!()));
    let owner = shittykittyz.owner_of(&router.wrap(), "3".to_string(), false).unwrap().owner;
    let owner2 = neonpeepz.owner_of(&router.wrap(), "3".to_string(), false).unwrap().owner;
    assert_eq!(owner, fuzionmarket.to_string());
    assert_eq!(owner2, fuzionmarket.to_string());

    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // Sam cannot remove John's listing
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

    let remove_john_1 = crate::msg::ExecuteMsg::DeleteListing {
        listing_id: 1,
    };

    let res: Result<AppResponse> =
        router.execute_contract(sam.address.clone(), fuzionmarket.clone(), &remove_john_1, &[]);
    ensure!(res.is_err(), here("sam fail remove", line!(), column!()));
    assert_eq!(jvtwo.balance(&router.wrap(), sam.address.clone()), Ok(Uint128::from(90u32)));
    assert_eq!(jvtwo.balance(&router.wrap(), fuzionmarket.clone()), Ok(Uint128::from(20u32)));

    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // John can remove his listing
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

    let res: Result<AppResponse> =
        router.execute_contract(john.address.clone(), fuzionmarket.clone(), &remove_john_1, &[]);
    ensure!(res.is_ok(), here("john remove", line!(), column!()));

    //~~~~~~~~~~~~~~~~~~~~~~~~~~
    // John's balance is updated
    // juno
    let john_new_balance: Coin =
        router.wrap().query_balance(john.address.to_string(), "ujunox").unwrap();
    ensure!(
        (john_new_balance.amount == Uint128::from(100_000_000_u32)),
        here(format!("John balance: {}", john_new_balance.amount), line!(), column!())
    );
    // jvone
    assert_eq!(jvone.balance(&router.wrap(), john.address.clone()), Ok(Uint128::from(100u32)));
    // jvtwo
    assert_eq!(jvtwo.balance(&router.wrap(), john.address.clone()), Ok(Uint128::from(100u32)));
    // John has his NFTs back
    let owner = shittykittyz.owner_of(&router.wrap(), "1".to_string(), false).unwrap().owner;
    assert_eq!(owner, john.address.clone().to_string());
    let owner2 = neonpeepz.owner_of(&router.wrap(), "1".to_string(), false).unwrap().owner;
    assert_eq!(owner2, john.address.clone().to_string());
    //~~~~~~~~~~~~~~~~~~~~~~~~~
    // Contract balance is updated
    // juno
    let contract_new_balance: Coin =
        router.wrap().query_balance(fuzionmarket.to_string(), "ujunox").unwrap();
    ensure!(
        (contract_new_balance.amount == Uint128::from(1u32)),
        here(format!("Contract balance: {}", contract_new_balance.amount), line!(), column!())
    );
    // jvone
    assert_eq!(jvone.balance(&router.wrap(), fuzionmarket.clone()), Ok(Uint128::from(10u32)));
    // jvtwo
    assert_eq!(jvtwo.balance(&router.wrap(), fuzionmarket.clone()), Ok(Uint128::from(10u32)));

    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // John's listing no longer exists
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

    let q = crate::msg::QueryMsg::GetListingsByOwner {
        owner: john.address.clone().to_string(),
    };

    let res: crate::query::MultiListingResponse =
        router.wrap().query_wasm_smart(fuzionmarket, &q).unwrap();

    ensure!(res.listings.is_empty(), here("john listings length", line!(), column!()));

    Ok(())
}

// Finalizing a listing
// <X> Expiration date verification <can't set expiration date over 2 weeks or less than 10 mins>
// <X> Can't finalize listing that's not your own
// <X> Can't finalize a listing that's already finalized
// <X> Can't remove a finalized listing
// <X> Can't refund a finalized listing that's not expired
// <X> Can't add to a finalized listing
#[test]
fn finalize_a_listing() -> Result<(), anyhow::Error> {
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // Finalize Listing checks
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    use anyhow::Result;
    use cw_multi_test::AppResponse;
    // Setup
    let mut router = App::default();
    let contract_admin = create_users::fake_user("admin".to_string());
    let john = create_users::fake_user("john".to_string());
    let sam = create_users::fake_user("sam".to_string());
    let max = create_users::fake_user("max".to_string());
    let bad_actor = create_users::fake_user("badguy".to_string());

    // Instantiate all contracts
    let (jvone, jvtwo, _jvtre, neonpeepz, shittykittyz, fuzionmarket) =
        init_all_contracts(&mut router, &contract_admin, &john, &sam, &max)?;
    // Gives each user
    // 100 JVONE, JVTWO, JVTRE
    // 2 ShittyKittyz + 2 NeonPeepz

    // Give each user 100 VALID_NATIVE
    let router = give_natives(&john, &mut router);
    let router = give_natives(&sam, router);
    let router = give_natives(&max, router);
    let router = give_natives(&bad_actor, router);

    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // Create 2 listings, add CW20's and NFTs
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    //~~~~~~~~~~
    // Listing one, created by John
    //~~~~~~~~~~
    let ask_price_valid = create_valid_listing::create_valid_ask(
        // Listing ID
        //1,
        // ujunox in ask
        Some(10),
        // jvone in ask
        Some(jvone.addr()),
        Some(Uint128::from(25u32)),
        // jvtwo in ask
        None,
        None,
        // jvtre in ask
        None,
        None,
        // NeonPeepz in ask
        None,
        None,
        // ShittyKittyz in ask
        None,
        None,
        // whitelisted purchasers
        None,
    );
    // For sale 1 ujunox
    let one_juno = coins(1, "ujunox");

    let res: Result<AppResponse> = router.execute_contract(
        john.address.clone(),
        fuzionmarket.clone(),
        &ask_price_valid,
        &one_juno,
    );
    // passes
    ensure!(res.is_ok(), here("'Testing Ask Creation' failure", line!(), column!()));
    let john_new_balance: Coin =
        router.wrap().query_balance(john.address.to_string(), "ujunox").unwrap();
    ensure!(
        (john_new_balance.amount == Uint128::from(99_999_999_u32)),
        here(format!("John balance: {}", john_new_balance.amount), line!(), column!())
    );

    //~~~~~~~~~~
    // Listing two, created by Sam
    //~~~~~~~~~~
    let ask_price_valid = create_valid_listing::create_valid_ask(
        // Listing ID
        //21,
        // ujunox in ask
        Some(10),
        // jvone in ask
        Some(jvone.addr()),
        Some(Uint128::from(25u32)),
        // jvtwo in ask
        None,
        None,
        // jvtre in ask
        None,
        None,
        // NeonPeepz in ask
        None,
        None,
        // ShittyKittyz in ask
        None,
        None,
        // whitelisted purchasers
        None,
    );
    // Listing for sale, 1 ujunox
    let one_juno = coins(1, "ujunox");
    let res: Result<AppResponse> = router.execute_contract(
        sam.address.clone(),
        fuzionmarket.clone(),
        &ask_price_valid,
        &one_juno,
    );
    // passes
    ensure!(res.is_ok(), here("'Testing Ask Creation' failure", line!(), column!()));
    //let john_new_balance = router.wrap().query_all_balances(addr).unwrap();
    let sam_new_balance: Coin =
        router.wrap().query_balance(sam.address.to_string(), "ujunox").unwrap();
    ensure!(
        (sam_new_balance.amount == Uint128::from(99_999_999_u32)),
        here(format!("Sam balance: {}", sam_new_balance.amount), line!(), column!())
    );

    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // Add 10 JVONE, 10 JVTWO to each
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

    let john_msg = to_binary(&crate::msg::ReceiveMsg::AddToListingCw20 {
        listing_id: 1,
    })?;
    let sam_msg = to_binary(&crate::msg::ReceiveMsg::AddToListingCw20 {
        listing_id: 2,
    })?;
    let john_add_ten_msg = cw20_base::msg::ExecuteMsg::Send {
        contract: fuzionmarket.to_string(),
        amount: Uint128::from(10u32),
        msg: john_msg,
    };
    let sam_add_ten_msg = cw20_base::msg::ExecuteMsg::Send {
        contract: fuzionmarket.to_string(),
        amount: Uint128::from(10u32),
        msg: sam_msg,
    };

    // John adding ten jvone
    let res: Result<AppResponse> =
        router.execute_contract(john.address.clone(), jvone.addr(), &john_add_ten_msg, &[]);
    ensure!(res.is_ok(), here("John added", line!(), column!()));
    assert_eq!(jvone.balance(&router.wrap(), john.address.clone()), Ok(Uint128::from(90u32)));
    assert_eq!(jvone.balance(&router.wrap(), fuzionmarket.clone()), Ok(Uint128::from(10u32)));
    // John adding ten jvtwo
    let res: Result<AppResponse> =
        router.execute_contract(john.address.clone(), jvtwo.addr(), &john_add_ten_msg, &[]);
    ensure!(res.is_ok(), here("Sam added", line!(), column!()));
    assert_eq!(jvtwo.balance(&router.wrap(), john.address.clone()), Ok(Uint128::from(90u32)));
    assert_eq!(jvtwo.balance(&router.wrap(), fuzionmarket.clone()), Ok(Uint128::from(10u32)));
    // ~~~
    // Sam adding ten jvone
    let res: Result<AppResponse> =
        router.execute_contract(sam.address.clone(), jvone.addr(), &sam_add_ten_msg, &[]);
    ensure!(res.is_ok(), here("Sam added", line!(), column!()));
    assert_eq!(jvone.balance(&router.wrap(), sam.address.clone()), Ok(Uint128::from(90u32)));
    assert_eq!(jvone.balance(&router.wrap(), fuzionmarket.clone()), Ok(Uint128::from(20u32)));
    // Sam adding ten jvtwo
    let res: Result<AppResponse> =
        router.execute_contract(sam.address.clone(), jvtwo.addr(), &sam_add_ten_msg, &[]);
    ensure!(res.is_ok(), here("sam added", line!(), column!()));
    assert_eq!(jvtwo.balance(&router.wrap(), sam.address.clone()), Ok(Uint128::from(90u32)));
    assert_eq!(jvtwo.balance(&router.wrap(), fuzionmarket.clone()), Ok(Uint128::from(20u32)));

    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // Adding 1 NeonPeep, 1 ShittyKitty to each listing
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

    let john_add_msg = to_binary(&crate::msg::ReceiveNftMsg::AddToListingCw721 {
        listing_id: 1,
    })?;
    let john_add_nft_msg: cw721_base::ExecuteMsg<Option<Empty>, Empty> =
        cw721_base::msg::ExecuteMsg::SendNft {
            contract: fuzionmarket.to_string(),
            token_id: "1".to_string(),
            msg: john_add_msg,
        };

    // John adding NeonPeepz 1 to his listing
    let res: Result<AppResponse> =
        router.execute_contract(john.address.clone(), neonpeepz.addr(), &john_add_nft_msg, &[]);
    ensure!(res.is_ok(), here("john", line!(), column!()));
    // John adding ShittyKittyz 1 to his listing
    let res: Result<AppResponse> =
        router.execute_contract(john.address.clone(), shittykittyz.addr(), &john_add_nft_msg, &[]);
    ensure!(res.is_ok(), here("john", line!(), column!()));
    // Contract has NFTs
    let owner = shittykittyz.owner_of(&router.wrap(), "1".to_string(), false).unwrap().owner;
    let owner2 = neonpeepz.owner_of(&router.wrap(), "1".to_string(), false).unwrap().owner;
    assert_eq!(owner, fuzionmarket.to_string());
    assert_eq!(owner2, fuzionmarket.to_string());

    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // Sam adding
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

    let sam_add_msg = to_binary(&crate::msg::ReceiveNftMsg::AddToListingCw721 {
        listing_id: 2,
    })?;
    let sam_add_nft_msg: cw721_base::ExecuteMsg<Option<Empty>, Empty> =
        cw721_base::msg::ExecuteMsg::SendNft {
            contract: fuzionmarket.to_string(),
            token_id: "3".to_string(),
            msg: sam_add_msg,
        };

    // Sam adding NeonPeepz 3 to her listing
    let res: Result<AppResponse> =
        router.execute_contract(sam.address.clone(), neonpeepz.addr(), &sam_add_nft_msg, &[]);
    ensure!(res.is_ok(), here("sam", line!(), column!()));
    // Sam adding ShittyKittyz 3 to her listing
    let res: Result<AppResponse> =
        router.execute_contract(sam.address.clone(), shittykittyz.addr(), &sam_add_nft_msg, &[]);
    ensure!(res.is_ok(), here("sam", line!(), column!()));
    let owner = shittykittyz.owner_of(&router.wrap(), "3".to_string(), false).unwrap().owner;
    let owner2 = neonpeepz.owner_of(&router.wrap(), "3".to_string(), false).unwrap().owner;
    assert_eq!(owner, fuzionmarket.to_string());
    assert_eq!(owner2, fuzionmarket.to_string());

    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // Sam cannot Finalize John's listing
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

    let finalize_john_1 = crate::msg::ExecuteMsg::Finalize {
        listing_id: 1,
        seconds: 259200,
    };

    let res: Result<AppResponse> =
        router.execute_contract(sam.address.clone(), fuzionmarket.clone(), &finalize_john_1, &[]);
    ensure!(res.is_err(), here("sam fail finalize", line!(), column!()));

    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // Invalid Expiration dates
    // max expiration is 1209600 seconds <14 days>
    // min expiration is 600 seconds <10 minutes>
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

    let too_early = crate::msg::ExecuteMsg::Finalize {
        listing_id: 1,
        seconds: 599,
    };
    let res: Result<AppResponse> =
        router.execute_contract(john.address.clone(), fuzionmarket.clone(), &too_early, &[]);
    ensure!(res.is_err(), here("Expiration too early", line!(), column!()));

    let too_late = crate::msg::ExecuteMsg::Finalize {
        listing_id: 1,
        seconds: 1209601,
    };
    let res: Result<AppResponse> =
        router.execute_contract(john.address.clone(), fuzionmarket.clone(), &too_late, &[]);
    ensure!(res.is_err(), here("expiration too late", line!(), column!()));

    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // Finalize works with valid expiration
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

    let just_right = crate::msg::ExecuteMsg::Finalize {
        listing_id: 1,
        seconds: 20000,
    };
    let res: Result<AppResponse> =
        router.execute_contract(john.address.clone(), fuzionmarket.clone(), &just_right, &[]);
    ensure!(res.is_ok(), here("Finalize with valid expiration failed", line!(), column!()));

    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // Can't finalize again
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    let finalize_again = crate::msg::ExecuteMsg::Finalize {
        listing_id: 1,
        seconds: 20000,
    };
    let res: Result<AppResponse> =
        router.execute_contract(john.address.clone(), fuzionmarket.clone(), &finalize_again, &[]);
    ensure!(res.is_err(), here("Finalize after finalize", line!(), column!()));

    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // Can't remove after finalize
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    let cant_remove = crate::msg::ExecuteMsg::DeleteListing {
        listing_id: 1,
    };
    let res: Result<AppResponse> =
        router.execute_contract(john.address.clone(), fuzionmarket.clone(), &cant_remove, &[]);
    ensure!(res.is_err(), here("Remove after finalize", line!(), column!()));

    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // Can't refund after finalize if not expired
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    let cant_refund = crate::msg::ExecuteMsg::DeleteListing {
        listing_id: 1,
    };
    let res: Result<AppResponse> =
        router.execute_contract(john.address.clone(), fuzionmarket.clone(), &cant_refund, &[]);
    ensure!(res.is_err(), here("Refund after finalize before expiration", line!(), column!()));

    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // Can't add after finalize
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // cant add native
    let cant_add = crate::msg::ExecuteMsg::AddToListing {
        listing_id: 1,
    };
    let res: Result<AppResponse> = router.execute_contract(
        john.address.clone(),
        fuzionmarket.clone(),
        &cant_add,
        &coins(1, "ujunox"),
    );
    ensure!(res.is_err(), here("add native after finalize", line!(), column!()));

    // cant add cw20
    let john_msg = to_binary(&crate::msg::ReceiveMsg::AddToListingCw20 {
        listing_id: 1,
    })?;
    let john_add_ten_msg = cw20_base::msg::ExecuteMsg::Send {
        contract: fuzionmarket.to_string(),
        amount: Uint128::from(10u32),
        msg: john_msg,
    };
    let res: Result<AppResponse> =
        router.execute_contract(john.address.clone(), jvone.addr(), &john_add_ten_msg, &[]);
    ensure!(res.is_err(), here("add cw20 after finalize", line!(), column!()));

    // cant add NFT
    let john_add_msg = to_binary(&crate::msg::ReceiveNftMsg::AddToListingCw721 {
        listing_id: 1,
    })?;
    let john_add_nft_msg: cw721_base::ExecuteMsg<Option<Empty>, Empty> =
        cw721_base::msg::ExecuteMsg::SendNft {
            contract: fuzionmarket.to_string(),
            token_id: "2".to_string(),
            msg: john_add_msg,
        };
    let res: Result<AppResponse> =
        router.execute_contract(john.address.clone(), neonpeepz.addr(), &john_add_nft_msg, &[]);
    ensure!(res.is_err(), here("add NFT after finalize", line!(), column!()));

    Ok(())
}

// Expiration Checks
// <X> Can't refund a listing that's not expired
// <X> Can't re-finalize a listing that's expired
// <X> Can't remove an expired listing <call refund instead>
// <X> Can't add to an expired listing
// <X> Refund a listing succeeds
#[test]
fn expiration_checks() -> Result<(), anyhow::Error> {
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // Expiration checks
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    use anyhow::Result;
    use cw_multi_test::AppResponse;
    // Setup
    let mut router = App::default();
    let contract_admin = create_users::fake_user("admin".to_string());
    let john = create_users::fake_user("john".to_string());
    let sam = create_users::fake_user("sam".to_string());
    let max = create_users::fake_user("max".to_string());
    let bad_actor = create_users::fake_user("badguy".to_string());

    // Instantiate all contracts
    let (jvone, jvtwo, _jvtre, neonpeepz, shittykittyz, fuzionmarket) =
        init_all_contracts(&mut router, &contract_admin, &john, &sam, &max)?;
    // Gives each user
    // 100 JVONE, JVTWO, JVTRE
    // 2 ShittyKittyz + 2 NeonPeepz

    // Give each user 100 VALID_NATIVE
    let router = give_natives(&john, &mut router);
    let router = give_natives(&sam, router);
    let router = give_natives(&max, router);
    let router = give_natives(&bad_actor, router);

    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // Create 2 listings, add CW20's and NFTs
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    //~~~~~~~~~~
    // Listing one, created by John
    //~~~~~~~~~~
    let ask_price_valid = create_valid_listing::create_valid_ask(
        // Listing ID
        //1,
        // ujunox in ask
        Some(10),
        // jvone in ask
        Some(jvone.addr()),
        Some(Uint128::from(25u32)),
        // jvtwo in ask
        None,
        None,
        // jvtre in ask
        None,
        None,
        // NeonPeepz in ask
        None,
        None,
        // ShittyKittyz in ask
        None,
        None,
        // whitelisted purchasers
        None,
    );
    // For sale 1 ujunox
    let one_juno = coins(1, "ujunox");

    let res: Result<AppResponse> = router.execute_contract(
        john.address.clone(),
        fuzionmarket.clone(),
        &ask_price_valid,
        &one_juno,
    );
    // passes
    ensure!(res.is_ok(), here("'Testing Ask Creation' failure", line!(), column!()));
    let john_new_balance: Coin =
        router.wrap().query_balance(john.address.to_string(), "ujunox").unwrap();
    ensure!(
        (john_new_balance.amount == Uint128::from(99_999_999_u32)),
        here(format!("John balance: {}", john_new_balance.amount), line!(), column!())
    );

    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // Add 10 JVONE, 10 JVTWO to each
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

    let john_msg = to_binary(&crate::msg::ReceiveMsg::AddToListingCw20 {
        listing_id: 1,
    })?;
    let john_add_ten_msg = cw20_base::msg::ExecuteMsg::Send {
        contract: fuzionmarket.to_string(),
        amount: Uint128::from(10u32),
        msg: john_msg,
    };

    // John adding ten jvone
    let res: Result<AppResponse> =
        router.execute_contract(john.address.clone(), jvone.addr(), &john_add_ten_msg, &[]);
    ensure!(res.is_ok(), here("John added", line!(), column!()));
    assert_eq!(jvone.balance(&router.wrap(), john.address.clone()), Ok(Uint128::from(90u32)));
    assert_eq!(jvone.balance(&router.wrap(), fuzionmarket.clone()), Ok(Uint128::from(10u32)));
    // John adding ten jvtwo
    let res: Result<AppResponse> =
        router.execute_contract(john.address.clone(), jvtwo.addr(), &john_add_ten_msg, &[]);
    ensure!(res.is_ok(), here("Sam added", line!(), column!()));
    assert_eq!(jvtwo.balance(&router.wrap(), john.address.clone()), Ok(Uint128::from(90u32)));
    assert_eq!(jvtwo.balance(&router.wrap(), fuzionmarket.clone()), Ok(Uint128::from(10u32)));

    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // Adding 1 NeonPeep, 1 ShittyKitty to each listing
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

    let john_add_msg = to_binary(&crate::msg::ReceiveNftMsg::AddToListingCw721 {
        listing_id: 1,
    })?;
    let john_add_nft_msg: cw721_base::ExecuteMsg<Option<Empty>, Empty> =
        cw721_base::msg::ExecuteMsg::SendNft {
            contract: fuzionmarket.to_string(),
            token_id: "1".to_string(),
            msg: john_add_msg,
        };

    // John adding NeonPeepz 1 to his listing
    let res: Result<AppResponse> =
        router.execute_contract(john.address.clone(), neonpeepz.addr(), &john_add_nft_msg, &[]);
    ensure!(res.is_ok(), here("john", line!(), column!()));
    // John adding ShittyKittyz 1 to his listing
    let res: Result<AppResponse> =
        router.execute_contract(john.address.clone(), shittykittyz.addr(), &john_add_nft_msg, &[]);
    ensure!(res.is_ok(), here("john", line!(), column!()));
    // Contract has NFTs
    let owner = shittykittyz.owner_of(&router.wrap(), "1".to_string(), false).unwrap().owner;
    let owner2 = neonpeepz.owner_of(&router.wrap(), "1".to_string(), false).unwrap().owner;
    assert_eq!(owner, fuzionmarket.to_string());
    assert_eq!(owner2, fuzionmarket.to_string());

    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // Finalize works with valid expiration
    // max expiration is 1209600 seconds <14 days>
    // min expiration is 600 seconds <10 minutes>
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

    let just_right = crate::msg::ExecuteMsg::Finalize {
        listing_id: 1,
        seconds: 20000,
    };
    let res: Result<AppResponse> =
        router.execute_contract(john.address.clone(), fuzionmarket.clone(), &just_right, &[]);
    ensure!(res.is_ok(), here("Finalize with valid expiration failed", line!(), column!()));

    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // Fast forward to listing is expired
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

    // Can't refund a listing that's not expired | 1 second before listing expires, should fail
    router.update_block(|current_blockinfo| {
        current_blockinfo.height += 4000;
        current_blockinfo.time = current_blockinfo.time.plus_seconds(19999);
    });
    let fail_refund = crate::msg::ExecuteMsg::DeleteListing {
        listing_id: 1,
    };
    let res: Result<AppResponse> =
        router.execute_contract(john.address.clone(), fuzionmarket.clone(), &fail_refund, &[]);
    ensure!(res.is_err(), here("Early Refund failure", line!(), column!()));

    // Add 1 second, Listing is now expired
    router.update_block(|current_blockinfo| {
        current_blockinfo.height += 1;
        current_blockinfo.time = current_blockinfo.time.plus_seconds(1);
    });

    // ~~~~~~~~~~~~
    // Cant refinalize an expired listing
    let fail_refinalize_expired = crate::msg::ExecuteMsg::Finalize {
        listing_id: 1,
        seconds: 15000,
    };
    let res: Result<AppResponse> = router.execute_contract(
        john.address.clone(),
        fuzionmarket.clone(),
        &fail_refinalize_expired,
        &[],
    );
    ensure!(res.is_err(), here("Refinalize expired failure", line!(), column!()));

    // ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~Remove for new Delete Listing Call
    // Can't remove an expired listing <must call refund instead>
    // let fail_remove = crate::msg::ExecuteMsg::DeleteListing {
    //     listing_id: 1,
    // };
    // let res: Result<AppResponse> =
    //     router.execute_contract(john.address.clone(), fuzionmarket.clone(), &fail_remove, &[]);
    // ensure!(res.is_err(), here("Remove expired failure", line!(), column!()));

    // ~~~~~~~~~
    // Can't add to expired listing
    let john_msg = to_binary(&crate::msg::ReceiveMsg::AddToListingCw20 {
        listing_id: 1,
    })?;
    let john_add_ten_msg = cw20_base::msg::ExecuteMsg::Send {
        contract: fuzionmarket.to_string(),
        amount: Uint128::from(10u32),
        msg: john_msg,
    };
    let res: Result<AppResponse> =
        router.execute_contract(john.address.clone(), jvone.addr(), &john_add_ten_msg, &[]);
    ensure!(res.is_err(), here("can't add to expired listing", line!(), column!()));

    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // Refund success
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    let success_refund = crate::msg::ExecuteMsg::DeleteListing {
        listing_id: 1,
    };
    let res: Result<AppResponse> =
        router.execute_contract(john.address.clone(), fuzionmarket.clone(), &success_refund, &[]);
    ensure!(res.is_ok(), here("Refund failure", line!(), column!()));

    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // Listing is deleted
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

    let q = crate::msg::QueryMsg::GetListingsByOwner {
        owner: john.address.clone().to_string(),
    };
    let res: crate::query::MultiListingResponse =
        router.wrap().query_wasm_smart(fuzionmarket, &q).unwrap();
    ensure!(res.listings.is_empty(), here("john listings length", line!(), column!()));

    Ok(())
}

//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// Buckets
//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

// Creating a Bucket
// <X> Create with each type <Native, CW20, NFT>
// <X> Add each type <Native, CW20, NFT>
// <X> Only owner can add
// <X> Removal
// <X> Bucket is deleted
#[test]
fn create_bucket() -> Result<(), anyhow::Error> {
    //use std::borrow::BorrowMut;
    use anyhow::Result;
    use cw_multi_test::AppResponse;
    // Setup
    let mut router = App::default();
    let contract_admin = create_users::fake_user("admin".to_string());
    let john = create_users::fake_user("john".to_string());
    let sam = create_users::fake_user("sam".to_string());
    let max = create_users::fake_user("max".to_string());
    let bad_actor = create_users::fake_user("badguy".to_string());

    // Instantiate all contracts
    let (jvone, _jvtwo, _jvtre, neonpeepz, _shittykittyz, fuzionmarket) =
        init_all_contracts(&mut router, &contract_admin, &john, &sam, &max)?;

    // Give native balances to all users
    // Each user gets 100 VALID_NATIVE
    let router = give_natives(&john, &mut router);
    let router = give_natives(&sam, router);
    let router = give_natives(&max, router);
    let router = give_natives(&bad_actor, router);

    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // Create with Native
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

    let create_native = crate::msg::ExecuteMsg::CreateBucket {};
    let res: Result<AppResponse> = router.execute_contract(
        john.address.clone(),
        fuzionmarket.clone(),
        &create_native,
        &coins(1, "ujunox"),
    );
    ensure!(res.is_ok(), here("Create Bucket native", line!(), column!()));
    let john_new_balance: Coin =
        router.wrap().query_balance(john.address.to_string(), "ujunox").unwrap();
    ensure!(
        (john_new_balance.amount == Uint128::from(99_999_999_u32)),
        here(format!("John balance: {}", john_new_balance.amount), line!(), column!())
    );

    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // Create with CW20
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

    let john_msg = to_binary(&crate::msg::ReceiveMsg::CreateBucketCw20 {}).unwrap();
    let john_c_msg = cw20_base::msg::ExecuteMsg::Send {
        contract: fuzionmarket.to_string(),
        amount: Uint128::from(10u32),
        msg: john_msg,
    };

    let res: Result<AppResponse> =
        router.execute_contract(john.address.clone(), jvone.addr(), &john_c_msg, &[]);
    ensure!(res.is_ok(), here("John create bucket cw20", line!(), column!()));
    assert_eq!(jvone.balance(&router.wrap(), john.address.clone()), Ok(Uint128::from(90u32)));
    assert_eq!(jvone.balance(&router.wrap(), fuzionmarket.clone()), Ok(Uint128::from(10u32)));

    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // Create with NFT
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

    let john_nft_msg = to_binary(&crate::msg::ReceiveNftMsg::CreateBucketCw721 {}).unwrap();
    let john_nft_c_msg: cw721_base::ExecuteMsg<Option<Empty>, Empty> =
        cw721_base::ExecuteMsg::SendNft {
            contract: fuzionmarket.to_string(),
            token_id: "1".to_string(),
            msg: john_nft_msg,
        };
    let res: Result<AppResponse> =
        router.execute_contract(john.address.clone(), neonpeepz.addr(), &john_nft_c_msg, &[]);
    ensure!(res.is_ok(), here("John create bucket NFT", line!(), column!()));
    let owner = neonpeepz.owner_of(&router.wrap(), "1".to_string(), false).unwrap().owner;
    assert_eq!(owner, fuzionmarket.to_string());

    //----------------------------------------------------------//

    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // Add Native
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    let john_add_msg = crate::msg::ExecuteMsg::AddToBucket {
        bucket_id: 3,
    };

    let res: Result<AppResponse> = router.execute_contract(
        john.address.clone(),
        fuzionmarket.clone(),
        &john_add_msg,
        &coins(1, "ujunox"),
    );
    ensure!(res.is_ok(), here("bucket add native", line!(), column!()));
    let john_new_balance: Coin =
        router.wrap().query_balance(john.address.to_string(), "ujunox").unwrap();
    ensure!(
        (john_new_balance.amount == Uint128::from(99_999_998_u32)),
        here(format!("John balance: {}", john_new_balance.amount), line!(), column!())
    );

    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // Add CW20
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

    let john_msg = to_binary(&crate::msg::ReceiveMsg::AddToBucketCw20 {
        bucket_id: 3,
    })
    .unwrap();
    let john_c_msg = cw20_base::msg::ExecuteMsg::Send {
        contract: fuzionmarket.to_string(),
        amount: Uint128::from(10u32),
        msg: john_msg,
    };

    let res: Result<AppResponse> =
        router.execute_contract(john.address.clone(), jvone.addr(), &john_c_msg, &[]);
    ensure!(res.is_ok(), here("John add bucket cw20", line!(), column!()));
    assert_eq!(jvone.balance(&router.wrap(), john.address.clone()), Ok(Uint128::from(80u32)));
    assert_eq!(jvone.balance(&router.wrap(), fuzionmarket.clone()), Ok(Uint128::from(20u32)));

    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // Add NFT
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

    let john_nft_msg = to_binary(&crate::msg::ReceiveNftMsg::AddToBucketCw721 {
        bucket_id: 3,
    })
    .unwrap();
    let john_nft_c_msg: cw721_base::ExecuteMsg<Option<Empty>, Empty> =
        cw721_base::ExecuteMsg::SendNft {
            contract: fuzionmarket.to_string(),
            token_id: "2".to_string(),
            msg: john_nft_msg,
        };

    let res: Result<AppResponse> =
        router.execute_contract(john.address.clone(), neonpeepz.addr(), &john_nft_c_msg, &[]);
    ensure!(res.is_ok(), here("John create bucket NFT", line!(), column!()));
    let owner2 = neonpeepz.owner_of(&router.wrap(), "2".to_string(), false).unwrap().owner;
    assert_eq!(owner2, fuzionmarket.to_string());

    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // Only owner can add
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

    let sam_fail = to_binary(&crate::msg::ReceiveNftMsg::AddToBucketCw721 {
        bucket_id: 3,
    })
    .unwrap();
    let sam_nft_fail: cw721_base::ExecuteMsg<Option<Empty>, Empty> =
        cw721_base::ExecuteMsg::SendNft {
            contract: fuzionmarket.to_string(),
            token_id: "3".to_string(),
            msg: sam_fail,
        };

    let res: Result<AppResponse> =
        router.execute_contract(sam.address.clone(), neonpeepz.addr(), &sam_nft_fail, &[]);
    ensure!(res.is_err(), here("Sam added to Johns bucket", line!(), column!()));

    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // Only owner can remove
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

    let remove = crate::msg::ExecuteMsg::RemoveBucket {
        bucket_id: 3,
    };
    let res: Result<AppResponse> =
        router.execute_contract(sam.address.clone(), fuzionmarket.clone(), &remove, &[]);
    ensure!(res.is_err(), here("Sam removed johns bucket", line!(), column!()));

    let res: Result<AppResponse> =
        router.execute_contract(john.address.clone(), fuzionmarket.clone(), &remove, &[]);
    ensure!(res.is_ok(), here("johns remove bucket", line!(), column!()));

    // balance checks
    let john_new_balance: Coin =
        router.wrap().query_balance(john.address.to_string(), "ujunox").unwrap();
    ensure!(
        (john_new_balance.amount == Uint128::from(99_999_999_u32)),
        here(format!("John balance: {}", john_new_balance.amount), line!(), column!())
    );

    assert_eq!(jvone.balance(&router.wrap(), john.address.clone()), Ok(Uint128::from(90u32)));
    assert_eq!(jvone.balance(&router.wrap(), fuzionmarket.clone()), Ok(Uint128::from(10u32)));

    let owner = neonpeepz.owner_of(&router.wrap(), "1".to_string(), false).unwrap().owner;
    assert_eq!(owner, john.address.clone().to_string());

    let owner2 = neonpeepz.owner_of(&router.wrap(), "2".to_string(), false).unwrap().owner;
    assert_eq!(owner2, john.address.clone().to_string());

    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // Bucket is deleted
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    let q = crate::msg::QueryMsg::GetBuckets {
        bucket_owner: john.address.clone().to_string(),
    };
    let res: crate::query::GetBucketsResponse =
        router.wrap().query_wasm_smart(fuzionmarket, &q).unwrap();

    ensure!((res.buckets.len() == 2), here("john buckets length", line!(), column!()));

    Ok(())
}

//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// Marketplace <Buying/Selling>
//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

// <X> Correct assets in Bucket required for purchase
// <X> Bucket creator cannot interact with Bucket after it's been used to purchase
// <X> Listing creator cannot interact with Listing after it's been sold
// <X> Purchased Listing only if they are whitelisted, if one is set
// <X> Bucket sale proceeds can only be removed once
// <X> Purchased Listing can only be removed once
// <X> Balance checks after Bucket Removal
// <X> Balance checks after Listing Removal
// <X> Fee is removed when withdrawing purchase
#[test]
fn marketplace_sale() -> Result<(), anyhow::Error> {
    use anyhow::Result;
    use cw_multi_test::AppResponse;
    // Setup
    let mut router = App::default();
    let contract_admin = create_users::fake_user("admin".to_string());
    let john = create_users::fake_user("john".to_string());
    let sam = create_users::fake_user("sam".to_string());
    let max = create_users::fake_user("max".to_string());
    let bad_actor = create_users::fake_user("badguy".to_string());

    // Instantiate all contracts
    let (jvone, jvtwo, jvtre, neonpeepz, shittykittyz, fuzionmarket) =
        init_all_contracts(&mut router, &contract_admin, &john, &sam, &max)?;

    // Give native balances to all users
    // Each user gets 100 VALID_NATIVE
    let router = give_natives(&john, &mut router);
    let router = give_natives(&sam, router);
    let router = give_natives(&max, router);
    let router = give_natives(&bad_actor, router);

    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // Create Listing
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // Create Listing Message
    // ASK PRICE
    // > 20 jvtwo
    // > shittykittyz #3
    // > Sam is whitelisted
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    let cw20_ask = vec![Cw20CoinVerified {
        address: jvtwo.addr(),
        amount: Uint128::from(20u32),
    }];
    let nft_ask = vec![Nft {
        contract_address: shittykittyz.addr(),
        token_id: "3".to_string(),
    }];
    let ask_price = GenericBalance {
        native: vec![],
        cw20: cw20_ask,
        nfts: nft_ask,
    };
    let cl = CreateListingMsg {
        //id: 1,
        ask: ask_price.clone(),
        //whitelisted_purchasers: Some(vec![sam.address.to_string(), john.address.to_string()]),
        whitelisted_buyer: Some(sam.address.to_string()),
    };
    let clm = crate::msg::ExecuteMsg::CreateListing {
        create_msg: cl,
    };

    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // Create listing w/ FOR_SALE: 5 Juno
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    let res: Result<AppResponse> = router.execute_contract(
        john.address.clone(),
        fuzionmarket.clone(),
        &clm,
        &coins(5_000_000, "ujunox"),
    );
    ensure!(res.is_ok(), here("'Testing Ask Creation' failure", line!(), column!()));

    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // add FOR_SALE: 10 JVONE
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

    let john_msg = to_binary(&crate::msg::ReceiveMsg::AddToListingCw20 {
        listing_id: 1,
    })
    .unwrap();
    let john_c_msg = cw20_base::msg::ExecuteMsg::Send {
        contract: fuzionmarket.to_string(),
        amount: Uint128::from(10u32),
        msg: john_msg,
    };

    let res: Result<AppResponse> =
        router.execute_contract(john.address.clone(), jvone.addr(), &john_c_msg, &[]);
    ensure!(res.is_ok(), here("John add listing cw20", line!(), column!()));

    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // add FOR_SALE: NeonPeepz #1
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

    let john_nft_msg = to_binary(&crate::msg::ReceiveNftMsg::AddToListingCw721 {
        listing_id: 1,
    })
    .unwrap();
    let john_nft_c_msg: cw721_base::ExecuteMsg<Option<Empty>, Empty> =
        cw721_base::ExecuteMsg::SendNft {
            contract: fuzionmarket.to_string(),
            token_id: "1".to_string(),
            msg: john_nft_msg,
        };

    let res: Result<AppResponse> =
        router.execute_contract(john.address.clone(), neonpeepz.addr(), &john_nft_c_msg, &[]);
    ensure!(res.is_ok(), here("John add listing NFT", line!(), column!()));

    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // Finalize
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

    let finalize = crate::msg::ExecuteMsg::Finalize {
        listing_id: 1,
        seconds: 10000,
    };

    let res: Result<AppResponse> =
        router.execute_contract(john.address.clone(), fuzionmarket.clone(), &finalize, &[]);
    ensure!(res.is_ok(), here("John finalize", line!(), column!()));

    //~~~~~~~~~~~~~~~~~~~~
    //
    // Listing ID: 1
    //
    // PRICE: JVTWO 20, ShittyKittyz #3
    //
    // FOR_SALE: JUNO 5, JVONE 10, NeonPeepz #1
    //
    //~~~~~~~~~~~~~~~~~~~~

    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // Try to buy with wrong Assets in Bucket
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

    //~~~~~~~~~~~~~
    // Correct NFT Address, Wrong NFT ID
    // Correct CW20 address, Correct amount
    //~~~~~~~~~~~~~
    // Create with 20 JVTWO
    let sam_msg = to_binary(&crate::msg::ReceiveMsg::CreateBucketCw20 {}).unwrap();
    let sam_c_msg = cw20_base::msg::ExecuteMsg::Send {
        contract: fuzionmarket.to_string(),
        amount: Uint128::from(20u32),
        msg: sam_msg,
    };

    let res: Result<AppResponse> =
        router.execute_contract(sam.address.clone(), jvtwo.addr(), &sam_c_msg, &[]);
    ensure!(res.is_ok(), here("sam create bucket", line!(), column!()));
    // Add ShittyKittyz #4 <Listing price is ShittyKittyz #3>
    let sam_nft_msg = to_binary(&crate::msg::ReceiveNftMsg::AddToBucketCw721 {
        bucket_id: 1,
    })
    .unwrap();
    let sam_nft_c_msg: cw721_base::ExecuteMsg<Option<Empty>, Empty> =
        cw721_base::ExecuteMsg::SendNft {
            contract: fuzionmarket.to_string(),
            token_id: "4".to_string(),
            msg: sam_nft_msg,
        };

    let res: Result<AppResponse> =
        router.execute_contract(sam.address.clone(), shittykittyz.addr(), &sam_nft_c_msg, &[]);
    ensure!(res.is_ok(), here("sam add NFT", line!(), column!()));

    // Try to buy listing, should fail
    let buy_msg = crate::msg::ExecuteMsg::BuyListing {
        listing_id: 1,
        bucket_id: 1,
    };
    let res: Result<AppResponse> =
        router.execute_contract(sam.address.clone(), fuzionmarket.clone(), &buy_msg, &[]);
    ensure!(res.is_err(), here("Sam buy listing wrong bucket", line!(), column!()));

    // Remove bucket
    let rem = crate::msg::ExecuteMsg::RemoveBucket {
        bucket_id: 1,
    };
    let res: Result<AppResponse> =
        router.execute_contract(sam.address.clone(), fuzionmarket.clone(), &rem, &[]);
    ensure!(res.is_ok(), here("sam remove bucket wrong", line!(), column!()));

    //~~~~~~~~~~~~~
    // Wrong NFT Address, Correct NFT ID
    // Correct CW20 address, Correct amount
    //~~~~~~~~~~~~~

    // Create with 20 JVTWO
    let sam_msg = to_binary(&crate::msg::ReceiveMsg::CreateBucketCw20 {}).unwrap();
    let sam_c_msg = cw20_base::msg::ExecuteMsg::Send {
        contract: fuzionmarket.to_string(),
        amount: Uint128::from(20u32),
        msg: sam_msg,
    };

    let res: Result<AppResponse> =
        router.execute_contract(sam.address.clone(), jvtwo.addr(), &sam_c_msg, &[]);
    ensure!(res.is_ok(), here("sam create bucket", line!(), column!()));
    // Add NeonPeepz #3 <Listing price is ShittyKittyz #3>
    let sam_nft_msg = to_binary(&crate::msg::ReceiveNftMsg::AddToBucketCw721 {
        bucket_id: 2,
    })
    .unwrap();
    let sam_nft_c_msg: cw721_base::ExecuteMsg<Option<Empty>, Empty> =
        cw721_base::ExecuteMsg::SendNft {
            contract: fuzionmarket.to_string(),
            token_id: "3".to_string(),
            msg: sam_nft_msg,
        };

    let res: Result<AppResponse> =
        router.execute_contract(sam.address.clone(), neonpeepz.addr(), &sam_nft_c_msg, &[]);
    ensure!(res.is_ok(), here("sam add NFT", line!(), column!()));

    // Try to buy listing, should fail
    let buy_msg = crate::msg::ExecuteMsg::BuyListing {
        listing_id: 1,
        bucket_id: 2,
    };
    let res: Result<AppResponse> =
        router.execute_contract(sam.address.clone(), fuzionmarket.clone(), &buy_msg, &[]);
    ensure!(res.is_err(), here("Sam buy listing wrong bucket", line!(), column!()));

    // Remove bucket
    let rem = crate::msg::ExecuteMsg::RemoveBucket {
        bucket_id: 2,
    };
    let res: Result<AppResponse> =
        router.execute_contract(sam.address.clone(), fuzionmarket.clone(), &rem, &[]);
    ensure!(res.is_ok(), here("sam remove bucket wrong", line!(), column!()));

    //~~~~~~~~~~~~~
    // Correct NFT Address, Correct NFT ID
    // Wrong CW20 address, Correct amount
    //~~~~~~~~~~~~~

    // Create with 20 JVTRE <Listing price is 20 JVTWO>
    let sam_msg = to_binary(&crate::msg::ReceiveMsg::CreateBucketCw20 {}).unwrap();
    let sam_c_msg = cw20_base::msg::ExecuteMsg::Send {
        contract: fuzionmarket.to_string(),
        amount: Uint128::from(20u32),
        msg: sam_msg,
    };

    let res: Result<AppResponse> =
        router.execute_contract(sam.address.clone(), jvtre.addr(), &sam_c_msg, &[]);
    ensure!(res.is_ok(), here("sam create bucket", line!(), column!()));
    // Add ShittyKittyz #3 <Listing price is ShittyKittyz #3>
    let sam_nft_msg = to_binary(&crate::msg::ReceiveNftMsg::AddToBucketCw721 {
        bucket_id: 3,
    })
    .unwrap();
    let sam_nft_c_msg: cw721_base::ExecuteMsg<Option<Empty>, Empty> =
        cw721_base::ExecuteMsg::SendNft {
            contract: fuzionmarket.to_string(),
            token_id: "3".to_string(),
            msg: sam_nft_msg,
        };

    let res: Result<AppResponse> =
        router.execute_contract(sam.address.clone(), shittykittyz.addr(), &sam_nft_c_msg, &[]);
    ensure!(res.is_ok(), here("sam add NFT", line!(), column!()));

    // Try to buy listing, should fail
    let buy_msg = crate::msg::ExecuteMsg::BuyListing {
        listing_id: 1,
        bucket_id: 3,
    };
    let res: Result<AppResponse> =
        router.execute_contract(sam.address.clone(), fuzionmarket.clone(), &buy_msg, &[]);
    ensure!(res.is_err(), here("Sam buy listing wrong bucket", line!(), column!()));

    // Remove bucket
    let rem = crate::msg::ExecuteMsg::RemoveBucket {
        bucket_id: 3,
    };
    let res: Result<AppResponse> =
        router.execute_contract(sam.address.clone(), fuzionmarket.clone(), &rem, &[]);
    ensure!(res.is_ok(), here("sam remove bucket wrong", line!(), column!()));

    //~~~~~~~~~~~~~
    // Correct NFT address, Correct NFT ID
    // Correct CW20 address, Wrong amount
    //~~~~~~~~~~~~~
    // Create with 19 JVTWO <Listing price is 20 JVTWO>
    let sam_msg = to_binary(&crate::msg::ReceiveMsg::CreateBucketCw20 {}).unwrap();
    let sam_c_msg = cw20_base::msg::ExecuteMsg::Send {
        contract: fuzionmarket.to_string(),
        amount: Uint128::from(19u32),
        msg: sam_msg,
    };

    let res: Result<AppResponse> =
        router.execute_contract(sam.address.clone(), jvtwo.addr(), &sam_c_msg, &[]);
    ensure!(res.is_ok(), here("sam create bucket", line!(), column!()));
    // Add ShittyKittyz #3 <Listing price is ShittyKittyz #3>
    let sam_nft_msg = to_binary(&crate::msg::ReceiveNftMsg::AddToBucketCw721 {
        bucket_id: 4,
    })
    .unwrap();
    let sam_nft_c_msg: cw721_base::ExecuteMsg<Option<Empty>, Empty> =
        cw721_base::ExecuteMsg::SendNft {
            contract: fuzionmarket.to_string(),
            token_id: "3".to_string(),
            msg: sam_nft_msg,
        };

    let res: Result<AppResponse> =
        router.execute_contract(sam.address.clone(), shittykittyz.addr(), &sam_nft_c_msg, &[]);
    ensure!(res.is_ok(), here("sam add NFT", line!(), column!()));

    // Try to buy listing, should fail
    let buy_msg = crate::msg::ExecuteMsg::BuyListing {
        listing_id: 1,
        bucket_id: 4,
    };
    let res: Result<AppResponse> =
        router.execute_contract(sam.address.clone(), fuzionmarket.clone(), &buy_msg, &[]);
    ensure!(res.is_err(), here("Sam buy listing wrong bucket", line!(), column!()));

    // Remove bucket
    let rem = crate::msg::ExecuteMsg::RemoveBucket {
        bucket_id: 4,
    };
    let res: Result<AppResponse> =
        router.execute_contract(sam.address.clone(), fuzionmarket.clone(), &rem, &[]);
    ensure!(res.is_ok(), here("sam remove bucket wrong", line!(), column!()));

    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // Correct bucket values but Max (not whitelisted) tries to buy
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

    // Create with 20 JVTWO <Listing price is 20 JVTWO>
    let max_msg = to_binary(&crate::msg::ReceiveMsg::CreateBucketCw20 {}).unwrap();
    let max_c_msg = cw20_base::msg::ExecuteMsg::Send {
        contract: fuzionmarket.clone().to_string(),
        amount: Uint128::from(20u32),
        msg: max_msg,
    };

    let res: Result<AppResponse> =
        router.execute_contract(max.address.clone(), jvtwo.addr(), &max_c_msg, &[]);
    ensure!(res.is_ok(), here("max create bucket correct", line!(), column!()));

    // Try to buy listing not whitelisted for, should fail
    let buy_msg = crate::msg::ExecuteMsg::BuyListing {
        listing_id: 1,
        bucket_id: 5,
    };
    let res: Result<AppResponse> =
        router.execute_contract(max.address.clone(), fuzionmarket.clone(), &buy_msg, &[]);
    ensure!(res.is_err(), here("Max tried to buy a listing not whitelisted", line!(), column!()));

    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~//

    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // Purchasing with correct bucket (Sam is whitelisted)
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

    // Create with 20 JVTWO <Listing price is 20 JVTWO>
    let sam_msg = to_binary(&crate::msg::ReceiveMsg::CreateBucketCw20 {}).unwrap();
    let sam_c_msg = cw20_base::msg::ExecuteMsg::Send {
        contract: fuzionmarket.to_string(),
        amount: Uint128::from(20u32),
        msg: sam_msg,
    };

    let res: Result<AppResponse> =
        router.execute_contract(sam.address.clone(), jvtwo.addr(), &sam_c_msg, &[]);
    ensure!(res.is_ok(), here("sam create bucket correct", line!(), column!()));
    // Add ShittyKittyz #3 <Listing price is ShittyKittyz #3>
    let sam_nft_msg = to_binary(&crate::msg::ReceiveNftMsg::AddToBucketCw721 {
        bucket_id: 6,
    })
    .unwrap();
    let sam_nft_c_msg: cw721_base::ExecuteMsg<Option<Empty>, Empty> =
        cw721_base::ExecuteMsg::SendNft {
            contract: fuzionmarket.to_string(),
            token_id: "3".to_string(),
            msg: sam_nft_msg,
        };

    let res: Result<AppResponse> =
        router.execute_contract(sam.address.clone(), shittykittyz.addr(), &sam_nft_c_msg, &[]);
    ensure!(res.is_ok(), here("sam add NFT", line!(), column!()));

    // Try to buy listing, should succeed
    let buy_msg = crate::msg::ExecuteMsg::BuyListing {
        listing_id: 1,
        bucket_id: 6,
    };
    let res: Result<AppResponse> =
        router.execute_contract(sam.address.clone(), fuzionmarket.clone(), &buy_msg, &[]);
    ensure!(res.is_ok(), here("Sam buy listing correct bucket", line!(), column!()));

    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // Sam can no longer remove bucket
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

    let rem = crate::msg::ExecuteMsg::RemoveBucket {
        bucket_id: 6,
    };
    let res: Result<AppResponse> =
        router.execute_contract(sam.address.clone(), fuzionmarket.clone(), &rem, &[]);
    ensure!(res.is_err(), here("sam remove bucket after purchase", line!(), column!()));

    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // But John can <listing seller>
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

    let res: Result<AppResponse> =
        router.execute_contract(john.address.clone(), fuzionmarket.clone(), &rem, &[]);
    ensure!(res.is_ok(), here("John remove bucket after purchase", line!(), column!()));

    // Balance check for John at end of function

    // Can't remove twice
    let res: Result<AppResponse> =
        router.execute_contract(john.address.clone(), fuzionmarket.clone(), &rem, &[]);
    ensure!(res.is_err(), here("John remove bucket after purchase", line!(), column!()));

    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // John can't do anything to listing
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // > Removed because Listing ID now come from state Incrementor <
    // Create listing with same name before it's removed should fail
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // let res: Result<AppResponse> = router.execute_contract(
    //     john.address.clone(),
    //     fuzionmarket.clone(),
    //     &clm,
    //     &coins(5_000_000, "ujunox"),
    // );
    // ensure!(res.is_err(), here(format!("xxx: {:#?}", res), line!(), column!()));

    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // John can't add
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    let john_msg = to_binary(&crate::msg::ReceiveMsg::AddToListingCw20 {
        listing_id: 1,
    })
    .unwrap();
    let john_c_msg = cw20_base::msg::ExecuteMsg::Send {
        contract: fuzionmarket.to_string(),
        amount: Uint128::from(10u32),
        msg: john_msg,
    };

    let res: Result<AppResponse> =
        router.execute_contract(john.address.clone(), jvone.addr(), &john_c_msg, &[]);
    ensure!(res.is_err(), here("John add after sale", line!(), column!()));

    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // John can't finalize
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    let finalize = crate::msg::ExecuteMsg::Finalize {
        listing_id: 1,
        seconds: 10000,
    };
    let res: Result<AppResponse> =
        router.execute_contract(john.address.clone(), fuzionmarket.clone(), &finalize, &[]);
    ensure!(res.is_err(), here("John finalize after sale", line!(), column!()));

    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // John can't Remove
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    let remove = crate::msg::ExecuteMsg::DeleteListing {
        listing_id: 1,
    };
    let res: Result<AppResponse> =
        router.execute_contract(john.address.clone(), fuzionmarket.clone(), &remove, &[]);
    ensure!(res.is_err(), here("John remove after sale", line!(), column!()));

    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // John can't Edit Price
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    let cw20_ask = vec![Cw20CoinVerified {
        address: jvtwo.addr(),
        amount: Uint128::from(20u32),
    }];
    let ask_price = GenericBalance {
        native: vec![],
        cw20: cw20_ask,
        nfts: vec![],
    };
    let edit_price = crate::msg::ExecuteMsg::ChangeAsk {
        listing_id: 1,
        new_ask: ask_price,
    };
    let res: Result<AppResponse> =
        router.execute_contract(john.address.clone(), fuzionmarket.clone(), &edit_price, &[]);
    ensure!(res.is_err(), here("John edit price after sale", line!(), column!()));

    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // Fast forward to after Listing Expiration date
    // to make sure John can't Refund
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    router.update_block(|current_blockinfo| {
        current_blockinfo.height += 10000;
        current_blockinfo.time = current_blockinfo.time.plus_seconds(60000);
    });

    let refund = crate::msg::ExecuteMsg::DeleteListing {
        listing_id: 1,
    };
    let res: Result<AppResponse> =
        router.execute_contract(john.address.clone(), fuzionmarket.clone(), &refund, &[]);
    ensure!(res.is_err(), here("John refund after sale", line!(), column!()));

    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // Edge case check that John cannot call WithdrawPurchased (he was the seller, not buyer)
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    let remove_edge = crate::msg::ExecuteMsg::WithdrawPurchased {
        listing_id: 1,
    };
    let res: Result<AppResponse> =
        router.execute_contract(john.address.clone(), fuzionmarket.clone(), &remove_edge, &[]);
    ensure!(res.is_err(), here("John withdraw after sale", line!(), column!()));

    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // Sam can remove the purchased listing
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    let res: Result<AppResponse> =
        router.execute_contract(sam.address.clone(), fuzionmarket.clone(), &remove_edge, &[]);
    ensure!(res.is_ok(), here(format!("{:#?}", res), line!(), column!()));

    // but can't remove twice
    let res: Result<AppResponse> =
        router.execute_contract(sam.address.clone(), fuzionmarket, &remove_edge, &[]);
    ensure!(res.is_err(), here("Sam Remove purchased twice", line!(), column!()));

    // PRICE: JVTWO 20, ShittyKittyz #3
    //
    // FOR_SALE: JUNO 5, JVONE 10, NeonPeepz #1

    // Sam balance checks
    // Sam should have
    // 105_000_000 JUNO before 0.1% fee
    // 0.1% of 5_000_000 is = 5_000
    // should have 104_995_000 JUNO
    // 110 JVONE
    // 80 JVTWO
    // NeonPeepz #1, #3, #4
    // ShittyKittyz #4
    let sam_juno_bal: Coin =
        router.wrap().query_balance(sam.address.to_string(), "ujunox").unwrap();
    ensure!(
        (sam_juno_bal.amount == Uint128::from(104_995_000_u32)),
        here("Sam juno balance wrong", line!(), column!())
    );

    assert_eq!(jvone.balance(&router.wrap(), sam.address.clone()), Ok(Uint128::from(110u32)));

    assert_eq!(jvtwo.balance(&router.wrap(), sam.address.clone()), Ok(Uint128::from(80u32)));

    let sam_neonpeepz =
        neonpeepz.tokens(&router.wrap(), sam.address.clone().to_string(), None, None).unwrap();
    assert!(sam_neonpeepz.tokens.contains(&"1".to_string()));
    assert!(sam_neonpeepz.tokens.contains(&"3".to_string()));
    assert!(sam_neonpeepz.tokens.contains(&"4".to_string()));
    assert_eq!(sam_neonpeepz.tokens.len(), 3);

    let sam_shittykittyz =
        shittykittyz.tokens(&router.wrap(), sam.address.clone().to_string(), None, None).unwrap();
    assert!(sam_shittykittyz.tokens.contains(&"4".to_string()));
    assert_eq!(sam_shittykittyz.tokens.len(), 1);

    // John balance checks
    // John should have
    // 95_000_000 JUNO
    // 90 JVONE
    // 120 JVTWO
    // NeonPeepz #2
    // ShittyKittyz #1, #2, #3
    let john_juno_bal: Coin =
        router.wrap().query_balance(john.address.to_string(), "ujunox").unwrap();
    ensure!(
        (john_juno_bal.amount == Uint128::from(95_000_000_u32)),
        here("John juno balance wrong", line!(), column!())
    );

    assert_eq!(jvone.balance(&router.wrap(), john.address.clone()), Ok(Uint128::from(90u32)));
    assert_eq!(jvtwo.balance(&router.wrap(), john.address.clone()), Ok(Uint128::from(120u32)));

    let john_neonpeepz =
        neonpeepz.tokens(&router.wrap(), john.address.clone().to_string(), None, None).unwrap();
    assert!(john_neonpeepz.tokens.contains(&"2".to_string()));
    assert_eq!(john_neonpeepz.tokens.len(), 1);

    let john_shittykittyz =
        shittykittyz.tokens(&router.wrap(), john.address.clone().to_string(), None, None).unwrap();
    assert!(john_shittykittyz.tokens.contains(&"1".to_string()));
    assert!(john_shittykittyz.tokens.contains(&"2".to_string()));
    assert!(john_shittykittyz.tokens.contains(&"3".to_string()));
    assert_eq!(john_shittykittyz.tokens.len(), 3);

    Ok(())
}

// <X> Expired listing cannot be purchased, even if Bucket has correct assets
#[test]
fn cant_buy_expired() -> Result<(), anyhow::Error> {
    use anyhow::Result;
    use cw_multi_test::AppResponse;
    // Setup
    let mut router = App::default();
    let contract_admin = create_users::fake_user("admin".to_string());
    let john = create_users::fake_user("john".to_string());
    let sam = create_users::fake_user("sam".to_string());
    let max = create_users::fake_user("max".to_string());
    let bad_actor = create_users::fake_user("badguy".to_string());

    // Instantiate all contracts
    let (jvone, jvtwo, _jvtre, neonpeepz, shittykittyz, fuzionmarket) =
        init_all_contracts(&mut router, &contract_admin, &john, &sam, &max)?;

    // Give native balances to all users
    // Each user gets 100 VALID_NATIVE
    let router = give_natives(&john, &mut router);
    let router = give_natives(&sam, router);
    let router = give_natives(&max, router);
    let router = give_natives(&bad_actor, router);

    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // Create Listing
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // Create Listing Message
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    let cw20_ask = vec![Cw20CoinVerified {
        address: jvtwo.addr(),
        amount: Uint128::from(20u32),
    }];
    let nft_ask = vec![Nft {
        contract_address: shittykittyz.addr(),
        token_id: "3".to_string(),
    }];
    let ask_price = GenericBalance {
        native: vec![],
        cw20: cw20_ask,
        nfts: nft_ask,
    };
    let cl = CreateListingMsg {
        //id: 1,
        ask: ask_price,
        //whitelisted_purchasers: None,
        whitelisted_buyer: None,
    };
    let clm = crate::msg::ExecuteMsg::CreateListing {
        create_msg: cl,
    };

    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // Create listing w/ FOR_SALE: 5 Juno
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    let res: Result<AppResponse> = router.execute_contract(
        john.address.clone(),
        fuzionmarket.clone(),
        &clm,
        &coins(5, "ujunox"),
    );
    ensure!(res.is_ok(), here("'Testing Ask Creation' failure", line!(), column!()));

    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // add FOR_SALE: 10 JVONE
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

    let john_msg = to_binary(&crate::msg::ReceiveMsg::AddToListingCw20 {
        listing_id: 1,
    })
    .unwrap();
    let john_c_msg = cw20_base::msg::ExecuteMsg::Send {
        contract: fuzionmarket.to_string(),
        amount: Uint128::from(10u32),
        msg: john_msg,
    };

    let res: Result<AppResponse> =
        router.execute_contract(john.address.clone(), jvone.addr(), &john_c_msg, &[]);
    ensure!(res.is_ok(), here("John add listing cw20", line!(), column!()));

    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // add FOR_SALE: NeonPeepz #1
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

    let john_nft_msg = to_binary(&crate::msg::ReceiveNftMsg::AddToListingCw721 {
        listing_id: 1,
    })
    .unwrap();
    let john_nft_c_msg: cw721_base::ExecuteMsg<Option<Empty>, Empty> =
        cw721_base::ExecuteMsg::SendNft {
            contract: fuzionmarket.to_string(),
            token_id: "1".to_string(),
            msg: john_nft_msg,
        };

    let res: Result<AppResponse> =
        router.execute_contract(john.address.clone(), neonpeepz.addr(), &john_nft_c_msg, &[]);
    ensure!(res.is_ok(), here("John add listing NFT", line!(), column!()));

    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // Finalize
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

    let finalize = crate::msg::ExecuteMsg::Finalize {
        listing_id: 1,
        seconds: 10000,
    };

    let res: Result<AppResponse> =
        router.execute_contract(john.address.clone(), fuzionmarket.clone(), &finalize, &[]);
    ensure!(res.is_ok(), here("John finalize", line!(), column!()));

    //~~~~~~~~~~~~~~~~~~~~
    // Listing ID: 1
    // PRICE: JVTWO 20, ShittyKittyz #3
    // FOR_SALE: JUNO 5, JVONE 10, NeonPeepz #1
    //~~~~~~~~~~~~~~~~~~~~

    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // Create bucket with correct assets
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

    // Create with 20 JVTWO <Listing price is 20 JVTWO>
    let sam_msg = to_binary(&crate::msg::ReceiveMsg::CreateBucketCw20 {}).unwrap();
    let sam_c_msg = cw20_base::msg::ExecuteMsg::Send {
        contract: fuzionmarket.to_string(),
        amount: Uint128::from(20u32),
        msg: sam_msg,
    };

    let res: Result<AppResponse> =
        router.execute_contract(sam.address.clone(), jvtwo.addr(), &sam_c_msg, &[]);
    ensure!(res.is_ok(), here("sam create bucket correct", line!(), column!()));
    // Add ShittyKittyz #3 <Listing price is ShittyKittyz #3>
    let sam_nft_msg = to_binary(&crate::msg::ReceiveNftMsg::AddToBucketCw721 {
        bucket_id: 1,
    })
    .unwrap();
    let sam_nft_c_msg: cw721_base::ExecuteMsg<Option<Empty>, Empty> =
        cw721_base::ExecuteMsg::SendNft {
            contract: fuzionmarket.to_string(),
            token_id: "3".to_string(),
            msg: sam_nft_msg,
        };

    let res: Result<AppResponse> =
        router.execute_contract(sam.address.clone(), shittykittyz.addr(), &sam_nft_c_msg, &[]);
    ensure!(res.is_ok(), here("sam add NFT", line!(), column!()));

    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // Fast forward to expiration
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

    // Fast forward to after Listing Expiration date
    router.update_block(|current_blockinfo| {
        current_blockinfo.height += 1667;
        current_blockinfo.time = current_blockinfo.time.plus_seconds(10001);
    });

    // Try to buy listing, should fail
    let buy_msg = crate::msg::ExecuteMsg::BuyListing {
        listing_id: 1,
        bucket_id: 1,
    };
    let res: Result<AppResponse> =
        router.execute_contract(sam.address.clone(), fuzionmarket, &buy_msg, &[]);
    ensure!(res.is_err(), here("Sam bought listing after expiration", line!(), column!()));

    Ok(())
}

// < > Storage capacity
#[test]
fn storage_capacity() -> Result<(), anyhow::Error> {
    use anyhow::Result;
    use cw_multi_test::AppResponse;
    // Setup
    let mut router = App::default();
    let contract_admin = create_users::fake_user("admin".to_string());
    let john = create_users::fake_user("john".to_string());
    let sam = create_users::fake_user("sam".to_string());
    let max = create_users::fake_user("max".to_string());
    let bad_actor = create_users::fake_user("badguy".to_string());

    // Instantiate all contracts
    let (_jvone, jvtwo, _jvtre, _neonpeepz, shittykittyz, fuzionmarket) =
        init_all_contracts(&mut router, &contract_admin, &john, &sam, &max)?;

    // Give native balances to all users
    // Each user gets 100 VALID_NATIVE
    let router = give_natives(&john, &mut router);
    let router = give_natives(&sam, router);
    let router = give_natives(&max, router);
    let router = give_natives(&bad_actor, router);

    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // Create Listing
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // Create Listing Message
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    let cw20_ask = vec![Cw20CoinVerified {
        address: jvtwo.addr(),
        amount: Uint128::from(20u32),
    }];
    let nft_ask = vec![Nft {
        contract_address: shittykittyz.addr(),
        token_id: "3".to_string(),
    }];
    let ask_price = GenericBalance {
        native: vec![],
        cw20: cw20_ask,
        nfts: nft_ask,
    };
    let cl = CreateListingMsg {
        //id: 1,
        ask: ask_price,
        //whitelisted_purchasers: None,
        whitelisted_buyer: None,
    };
    let clm = crate::msg::ExecuteMsg::CreateListing {
        create_msg: cl,
    };

    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // Create listing w/ FOR_SALE: 5 Juno
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

    let x = [1; 100];

    for _num in x {
        let res: Result<AppResponse> = router.execute_contract(
            john.address.clone(),
            fuzionmarket.clone(),
            &clm,
            &coins(1, "ujunox"),
        );

        ensure!(res.is_ok(), here("'Testing Ask Creation' failure", line!(), column!()));
    }

    for _num in x {
        let res: Result<AppResponse> = router.execute_contract(
            sam.address.clone(),
            fuzionmarket.clone(),
            &clm,
            &coins(1, "ujunox"),
        );
        ensure!(res.is_ok(), here("'Testing Ask Creation' failure", line!(), column!()));
    }

    for _num in x {
        let res: Result<AppResponse> = router.execute_contract(
            max.address.clone(),
            fuzionmarket.clone(),
            &clm,
            &coins(1, "ujunox"),
        );
        ensure!(res.is_ok(), here("'Testing Ask Creation' failure", line!(), column!()));
    }

    for _num in x {
        let res: Result<AppResponse> = router.execute_contract(
            bad_actor.address.clone(),
            fuzionmarket.clone(),
            &clm,
            &coins(1, "ujunox"),
        );
        ensure!(res.is_ok(), here("'Testing Ask Creation' failure", line!(), column!()));
    }

    Ok(())

    // let res: Result<AppResponse> = router.execute_contract(
    //     john.address.clone(),
    //     fuzionmarket.clone(),
    //     &clm,
    //     &coins(1, "ujunox"),
    // );
    // ensure!(res.is_ok(), here("'Testing Ask Creation' failure", line!(), column!()));
}
