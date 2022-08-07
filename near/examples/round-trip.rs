fn main() {}

#[cfg(test)]
mod test {
    use anyhow::Result;
    use near_units::parse_near;
    use std::time::SystemTime;
    use workspaces::prelude::*;
    use workspaces::{types::Balance, Account, AccountId, Contract, DevNetwork, Worker};

    async fn init(worker: &Worker<impl DevNetwork>) -> Result<Contract> {
        let wasm = std::fs::read("../target/wasm32-unknown-unknown/release/near_atomic_swap.wasm")?;
        let contract = worker.dev_deploy(&wasm).await?;
        Ok(contract)
    }

    async fn fund(
        worker: &Worker<impl DevNetwork>,
        contract: &Contract,
        caller: &Account,
        sender: &AccountId,
        receiver: &AccountId,
        amount: Balance,
        hashlock: [u8; 32],
        timelock: u64,
    ) -> Result<()> {
        let res = caller
            .call(&worker, contract.id(), "fund")
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
        caller: &Account,
        sender: &AccountId,
        receiver: &AccountId,
        amount: Balance,
        hashlock: [u8; 32],
        timelock: u64,
        secret_key: [u8; 32],
    ) -> Result<()> {
        let res = caller
            .call(&worker, contract.id(), "confirm")
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
        let account = worker.dev_create_account().await?;

        let sender = account
            .create_subaccount(&worker, "sender")
            .initial_balance(parse_near!("20 N"))
            .transact()
            .await?
            .into_result()?;
        let sender_id: AccountId = format!("sender.{}", account.id()).parse().unwrap();

        let sender_balance = sender.view_account(&worker).await?.balance;
        assert_eq!(sender_balance, 20_000_000_000_000_000_000_000_000);

        let receiver = account
            .create_subaccount(&worker, "receiver")
            .initial_balance(parse_near!("1 N"))
            .transact()
            .await?
            .into_result()?;
        let receiver_id: AccountId = format!("receiver.{}", account.id()).parse().unwrap();

        let receiver_balance = receiver.view_account(&worker).await?.balance;
        assert_eq!(receiver_balance, 1_000_000_000_000_000_000_000_000);

        let timelock = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let contract = init(&worker).await?;

        fund(
            &worker,
            &contract,
            &sender,
            &sender_id,
            &receiver_id,
            100,
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
            &receiver,
            &sender_id,
            &receiver_id,
            100,
            [
                165, 152, 132, 76, 216, 153, 182, 114, 45, 89, 20, 251, 170, 95, 204, 77, 214, 166,
                43, 58, 171, 243, 206, 181, 109, 46, 63, 177, 197, 13, 234, 154,
            ],
            timelock,
            *b"ssssssssssssssssssssssssssssssss",
        )
        .await?;

        let new_receiver_balance = receiver.view_account(&worker).await?.balance;

        assert_eq!(new_receiver_balance, receiver_balance + 100);

        Ok(())
    }
}
