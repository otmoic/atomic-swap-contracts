fn main() {}

#[cfg(test)]
mod test {
    use anyhow::Result;
    use std::time::SystemTime;
    use workspaces::prelude::*;
    use workspaces::{types::Balance, AccountId, Contract, DevNetwork, Worker};

    async fn init(worker: &Worker<impl DevNetwork>) -> Result<Contract> {
        let wasm = std::fs::read("../target/wasm32-unknown-unknown/release/near_atomic_swap.wasm")?;
        let contract = worker.dev_deploy(&wasm).await?;
        Ok(contract)
    }

    async fn fund(
        worker: &Worker<impl DevNetwork>,
        contract: &Contract,
        sender: AccountId,
        receiver: AccountId,
        amount: Balance,
        hashlock: [u8; 32],
        timelock: u64,
    ) -> Result<()> {
        let res = contract
            .call(&worker, "fund")
            .args_json((sender, receiver, amount, hashlock, timelock))?
            .gas(300_000_000_000_000)
            .deposit(2)
            .transact()
            .await?;
        assert!(res.is_success());

        Ok(())
    }

    async fn confirm(
        worker: &Worker<impl DevNetwork>,
        contract: &Contract,
        sender: AccountId,
        receiver: AccountId,
        amount: Balance,
        hashlock: [u8; 32],
        timelock: u64,
        secret_key: [u8; 32],
    ) -> Result<()> {
        let res = contract
            .call(&worker, "confirm")
            .args_json((sender, receiver, amount, hashlock, timelock, secret_key))?
            .gas(300_000_000_000_000)
            .transact()
            .await?;
        assert!(res.is_success());

        Ok(())
    }

    #[test_with::file("target/wasm32-unknown-unknown/release/near_atomic_swap.wasm")]
    #[tokio::test]
    async fn round_trip() -> Result<()> {
        let worker = workspaces::sandbox().await?;
        let timelock = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let contract = init(&worker).await?;

        fund(
            &worker,
            &contract,
            "caller".parse().unwrap(),
            "receiver".parse().unwrap(),
            1,
            [
                165, 152, 132, 76, 216, 153, 182, 114, 45, 89, 20, 251, 170, 95, 204, 77, 214, 166,
                43, 58, 171, 243, 206, 181, 109, 46, 63, 177, 197, 13, 234, 154,
            ],
            timelock,
        )
        .await?;

        confirm(
            &worker,
            &contract,
            "caller".parse().unwrap(),
            "receiver".parse().unwrap(),
            1,
            [
                165, 152, 132, 76, 216, 153, 182, 114, 45, 89, 20, 251, 170, 95, 204, 77, 214, 166,
                43, 58, 171, 243, 206, 181, 109, 46, 63, 177, 197, 13, 234, 154,
            ],
            timelock,
            *b"ssssssssssssssssssssssssssssssss",
        )
        .await?;

        Ok(())
    }
}
