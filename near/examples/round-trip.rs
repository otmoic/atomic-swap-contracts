fn main() {}

#[cfg(test)]
mod test {
    use anyhow::Result;
    use near_sdk::json_types::U128;
    use near_units::parse_near;
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

    async fn init(
        worker: &Worker<impl DevNetwork>,
        initial_balance: U128,
    ) -> Result<(Contract, Account)> {
        let wasm = std::fs::read("../target/wasm32-unknown-unknown/release/near_atomic_swap.wasm")?;
        let contract = worker.dev_deploy(&wasm).await?;
        let res = contract
            .call(&worker, "new_default_meta")
            .args_json((contract.id(), initial_balance))?
            .gas(300_000_000_000_000)
            .transact()
            .await?;
        assert!(res.is_success());

        let alice = contract
            .as_account()
            .create_subaccount(&worker, "alice")
            .initial_balance(parse_near!("10 N"))
            .transact()
            .await?
            .into_result()?;
        register_user(worker, &contract, alice.id()).await?;

        return Ok((contract, alice));
    }

    #[test_with::file("target/wasm32-unknown-unknown/release/near_atomic_swap.wasm")]
    #[tokio::test]
    async fn round_trip() -> Result<()> {
        let initial_balance = U128::from(parse_near!("10000 N"));
        let worker = workspaces::sandbox().await?;
        let (_contract, _alice) = init(&worker, initial_balance).await?;
        Ok(())
    }
}
