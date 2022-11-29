#[cfg(test)]
mod tests {
    use crate::helpers::DepositContract;
    use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, Cw20HookMsg, Cw20DepositResponse, DepositResponse};
    use cosmwasm_std::{Addr, Coin, Empty, Uint128, to_binary, coin, WasmMsg};
    use cw20::{Cw20Contract, Cw20Coin, BalanceResponse};
    use cw20_base::msg::ExecuteMsg as Cw20ExecuteMsg;
    use cw20_base::msg::InstantiateMsg as Cw20InstantiateMsg;
    use cw20_base::msg::QueryMsg as Cw20QueryMsg;
    use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};

    use cw20_example::{self};

    pub fn contract_deposit_cw20() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        );
        Box::new(contract)
    }

    pub fn contract_cw20() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            cw20_example::contract::execute,
            cw20_example::contract::instantiate,
            cw20_example::contract::query,
        );
        Box::new(contract)
    }

    const USER: &str = "juno10c3slrqx3369mfsr9670au22zvq082jaej8ve4";
    const ADMIN: &str = "ADMIN";
    const NATIVE_DENOM: &str = "denom";

    fn mock_app() -> App {
        AppBuilder::new().build(|router, _, storage| {
            router
                .bank
                .init_balance(
                    storage,
                    &Addr::unchecked(USER),
                    vec![Coin {
                        denom: NATIVE_DENOM.to_string(),
                        amount: Uint128::new(1000),
                    }],
                )
                .unwrap();
        })
    }

    fn store_code() -> (App, u64, u64) {
        let mut app = mock_app();
        
        let deposit_id = app.store_code(contract_deposit_cw20());
        let cw20_id = app.store_code(contract_cw20());
        (app, deposit_id, cw20_id)
    }

    fn deposit_instantiate(app: &mut App, deposit_id: u64) -> DepositContract {
        let msg = InstantiateMsg {};
        let deposit_contract_address = app
            .instantiate_contract(
                deposit_id,
                Addr::unchecked(ADMIN),
                &msg,
                &[],
                "deposit-cw20",
                None,
            )
            .unwrap();
        DepositContract(deposit_contract_address)
    }

    fn cw_20_instantiate(app: &mut App, cw20_id:u64) -> Cw20Contract {
        let coin = Cw20Coin {address:USER.to_string(), amount:Uint128::from(10000u64)};
        let msg:Cw20InstantiateMsg = Cw20InstantiateMsg {decimals:10, name:"Token".to_string(), symbol:"TKN".to_string(), initial_balances:vec![coin], marketing:None, mint:None };
        let cw20_contract_address = app
        .instantiate_contract(
            cw20_id,
            Addr::unchecked(ADMIN),
            &msg,
            &[],
            "cw20-example",
            None,
        )
        .unwrap();
    Cw20Contract(cw20_contract_address)
    }

    fn get_deposits(app: &App, deposit_contract: &DepositContract) -> DepositResponse {
        app.wrap()
            .query_wasm_smart(deposit_contract.addr(), &QueryMsg::Deposits { address: USER.to_string() })
            .unwrap()
    }

    fn get_balance(app: &App, user:String, denom:String) -> Coin {
        app.wrap().query_balance(user, denom).unwrap()
    }

    fn get_cw20_deposits(app: &App, deposit_contract: &DepositContract) -> Cw20DepositResponse {
        app.wrap()
            .query_wasm_smart(deposit_contract.addr(), &QueryMsg::Cw20Deposits { address: USER.to_string() })
            .unwrap()
    }

    // Gets the balance on the cw20 Contract, not this one.
    fn get_cw20_balance(app: &App, cw20_contract: &Cw20Contract, user:String) -> BalanceResponse {
        app.wrap()
            .query_wasm_smart(cw20_contract.addr(), &Cw20QueryMsg::Balance { address: user })
            .unwrap()
    }


    #[test]
    fn deposit_native() {
        let (mut app, deposit_id, cw20_id) = store_code();
        let deposit_contract = deposit_instantiate(&mut app, deposit_id);

        // The Blockchain was setup with an initial balance of (denom, 1000) for USER.
        let balance = get_balance(&app, USER.to_string(), "denom".to_string());
        println!("1. USER - Initial Balance # {:?} # ", balance);

        // User stores (denom, 1000) on deposit contract. The contract now owns the coins. User does not own any coins.
        let msg = ExecuteMsg::Deposit { };
        let cosmos_msg = deposit_contract.call(msg, vec![coin(1000, "denom")]).unwrap();
        app.execute(Addr::unchecked(USER), cosmos_msg).unwrap();

        let balance = get_balance(&app, USER.to_string(), "denom".to_string());
        println!("2. USER - Balance after 1000 deposit on deposit_contract # {:?} # ", balance);

        let balance = get_balance(&app, deposit_contract.addr().into_string(), "denom".into());
        println!("3. DEPOSIT CONTRACT - balance # {:?} # ", balance);

        let balance = get_deposits(&app, &deposit_contract);
        println!("4. USER BALANCE ON DEPOSIT CONTRACT - balance # {:?} # ", balance);

    }

    #[test]
    fn deposit_cw20() {
        let (mut app, deposit_id, cw20_id) = store_code();
        let deposit_contract = deposit_instantiate(&mut app, deposit_id);
        let cw20_contract = cw_20_instantiate(&mut app, cw20_id);

        // On instantiation, User gets 10000 of the cw20 tokens on the cw_20 contract.
        let balance = get_cw20_balance(&app, &cw20_contract, USER.to_string());
        println!("1. CW20 Contract- Initial Balance for USER # {:?}", balance);

        // The user sends 500 of those tokens to the deposit contract.
        let hook_msg = Cw20HookMsg::Deposit { };
        let msg = Cw20ExecuteMsg::Send { contract: deposit_contract.addr().to_string(), amount: Uint128::from(500u64), msg: to_binary(&hook_msg).unwrap() };
        let cosmos_msg = cw20_contract.call(msg).unwrap();
        app.execute(Addr::unchecked(USER), cosmos_msg).unwrap();

        // Now on the cw_20 contract the user has 9500
        let balance = get_cw20_balance(&app, &cw20_contract, USER.to_string());
        println!("2. CW20 Contract - USER balance after withdrawing 500 to deposit_contract # {:?}", balance);

        // But on the deposit contract there is some money. get_cw20_deposits querys on the deposit_contract the User Balance
        let deposits = get_cw20_deposits(&app, &deposit_contract);
        println!("3. DEPOSIT contract - User deposits {:?}", deposits.deposits[0]);

        let balance = get_cw20_balance(&app, &cw20_contract, deposit_contract.addr().into_string());
        println!("4. CW20 Contract - DEPOSIT CONTRACT balance # {:?}", balance);
        assert_eq!(Uint128::from(500u64), balance.balance);

        let balance = get_cw20_balance(&app, &cw20_contract, USER.to_string());
        println!("5. CW20 contract - User Balance {:?}", balance);

        let balance = get_deposits(&app, &deposit_contract);
        println!("6. DEPOSIT CONTRACT - USER balance # {:?} # ", balance);


    }


    #[test]
    fn deposit_cw20_and_withdraw_after_expiration_has_passed() {
        let (mut app, deposit_id, cw20_id) = store_code();
        let deposit_contract = deposit_instantiate(&mut app, deposit_id);
        let cw20_contract = cw_20_instantiate(&mut app, cw20_id);

        let balance = get_cw20_balance(&app, &cw20_contract, USER.to_string());
        println!("1. CW20 Contract- Initial Balance for USER # {:?}", balance);

        // 500 cw20 tokens are sent to the deposit contract. 
        // The CW20 contract registers that the deposit contract has the 500 tokens
        let hook_msg = Cw20HookMsg::Deposit { };
        let msg = Cw20ExecuteMsg::Send { contract: deposit_contract.addr().to_string(), amount: Uint128::from(500u64), msg: to_binary(&hook_msg).unwrap() };
        let cosmos_msg = cw20_contract.call(msg).unwrap();
        app.execute(Addr::unchecked(USER), cosmos_msg).unwrap();
        println!("SEND 500 FROM CW20 CONTRACT TO DEPOSIT CONTRACT FOR USER");
        
        let balance = get_cw20_balance(&app, &cw20_contract, USER.to_string());
        println!("2. CW20 Contract- USER  balance after 500 transfer to DEPOSIT CONTRACT# {:?}", balance);

        let balance = get_cw20_balance(&app, &cw20_contract, deposit_contract.addr().into_string());
        println!("3. CW20 Contract - DEPOSIT CONTRACT balance # {:?}", balance);

        let deposits = get_cw20_deposits(&app, &deposit_contract);
        println!("4. DEPOSIT contract - USER deposits {:?}", deposits.deposits[0]);

        let mut block = app.block_info(); 
        block.height = app.block_info().height.checked_add(20).unwrap();
        app.set_block(block);

        let msg = ExecuteMsg::WithdrawCw20 {address:cw20_contract.addr().to_string(), amount:Uint128::from(500u64)};
        let execute_msg = WasmMsg::Execute { contract_addr: deposit_contract.addr().to_string(), msg: to_binary(&msg).unwrap(), funds: vec![] };
        app.execute(Addr::unchecked(USER), execute_msg.into()).unwrap();

        println!("WITHDRAW 500 from DEPOSIT CONTRACT - KEEP THEN IN CW20 CONTRACT FOR USER");

        let balance = get_cw20_balance(&app, &cw20_contract, USER.to_string());
        println!("5. CW20 Contract- USER  balance # {:?}", balance);

        let balance = get_cw20_balance(&app, &cw20_contract, deposit_contract.addr().into_string());
        println!("6. CW20 Contract - DEPOSIT CONTRACT balance # {:?}", balance);

        let deposits = get_cw20_deposits(&app, &deposit_contract);
        println!("7. DEPOSIT contract - USER deposits {:?}", deposits.deposits[0]);       

    }
   
}
