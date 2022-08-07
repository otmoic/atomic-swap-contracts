fn main() {}

#[cfg(test)]
mod test {
    use anyhow::Result;
    use near_sdk::json_types::U128;
    use near_units::parse_near;
    use std::time::SystemTime;
    use workspaces::prelude::*;
    use workspaces::{Account, AccountId, Contract, DevNetwork, Network, Worker};

    async fn register_user(
        worker: &Worker<impl Network>,
        contract: &Contract,
        account_id: &AccountId,
    ) -> Result<()> {
        let res = contract
            .call(&worker, "storage_deposit")
            .args_json((account_id, Option::<bool>::None))?
            .gas(300_000_000_000_000)
            .deposit(near_sdk::env::storage_byte_cost() * 125)
            .transact()
            .await?;
        assert!(res.is_success());

        Ok(())
    }

    async fn init_fund(
        worker: &Worker<impl DevNetwork>,
        sender: AccountId,
        receiver: AccountId,
        amount: U128,
        hashlock: [u8; 32],
        timelock: u64,
    ) -> Result<Contract> {
        let wasm = std::fs::read("../target/wasm32-unknown-unknown/release/near_atomic_swap.wasm")?;
        let contract = worker.dev_deploy(&wasm).await?;
        let res = contract
            .call(&worker, "ping_pong")
            .args_json((sender, receiver, 
                        // amount,
                        hashlock, 
                        timelock))?
            .gas(300_000_000_000_000)
            .transact()
            .await?;
        assert!(res.is_success());
        let return_msg: String = res.json()?;
        assert_eq!(return_msg, "".to_string());

        return Ok(contract);
    }

    #[test_with::file("target/wasm32-unknown-unknown/release/near_atomic_swap.wasm")]
    #[tokio::test]
    async fn round_trip() -> Result<()> {
        let initial_balance = U128::from(parse_near!("10000 N"));
        let worker = workspaces::sandbox().await?;
        let _contract = init_fund(
            &worker,
            "sender".parse().unwrap(),
            "receiver".parse().unwrap(),
            initial_balance,
            [0; 32],
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        )
        .await?;
        Ok(())
    }
}
