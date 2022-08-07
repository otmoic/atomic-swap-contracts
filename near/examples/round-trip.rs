fn main() {}

#[cfg(test)]
mod test {
    use anyhow::Result;
    use near_units::parse_near;
    use std::time::SystemTime;
    use workspaces::prelude::*;
    use workspaces::{types::Balance, Account, AccountId, Contract, DevNetwork, Network, Worker};

    async fn init(worker: &Worker<impl DevNetwork>) -> Result<Contract> {
        let wasm = std::fs::read("../target/wasm32-unknown-unknown/release/near_atomic_swap.wasm")?;
        let contract = worker.dev_deploy(&wasm).await?;
        Ok(contract)
    }

    async fn fund(
        worker: &Worker<impl DevNetwork>,
        contract: Contract,
        sender: AccountId,
        receiver: AccountId,
        amount: Balance,
        hashlock: [u8; 32],
        timelock: u64,
    ) -> Result<Contract> {
        let res = contract
            .call(&worker, "fund")
            .args_json((sender, receiver, amount, hashlock, timelock))?
            .gas(300_000_000_000_000)
            .deposit(2)
            .transact()
            .await?;
        assert!(res.is_success());

        Ok(contract)
    }

    async fn confirm(
        worker: &Worker<impl DevNetwork>,
        contract: Contract,
        sender: AccountId,
        receiver: AccountId,
        amount: Balance,
        hashlock: [u8; 32],
        timelock: u64,
        secret_key: [u8; 32],
    ) -> Result<Contract> {
        let res = contract
            .call(&worker, "confirm")
            .args_json((sender, receiver, amount, hashlock, timelock, secret_key))?
            .gas(300_000_000_000_000)
            .transact()
            .await?;
        assert!(res.is_success());

        Ok(contract)
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

        let contract_detail = worker.view_account(contract.id()).await?;
        let contract_balance = contract_detail.balance;

        let contract = fund(
            &worker,
            contract,
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

        let contract_detail = worker.view_account(contract.id()).await?;
        assert_eq!(contract_detail.balance, contract_balance + 2);

        confirm(
            &worker,
            contract,
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

        let receiver_account: AccountId = "receiver".parse().unwrap();
        let account_detail = worker.view_account(&receiver_account).await?;
        assert_eq!(account_detail.balance, 2);

        Ok(())
    }
}
