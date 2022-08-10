fn main() {}

#[cfg(test)]
mod test {
    use near_atomic_swap::TransferId;

    use anyhow::Result;
    use near_units::parse_near;
    use std::time::SystemTime;
    use workspaces::prelude::*;
    use workspaces::{network::Sandbox, types::Balance, Account, AccountId, Contract, Worker};

    async fn init(worker: &Worker<Sandbox>) -> Result<Contract> {
        let wasm = std::fs::read("../target/wasm32-unknown-unknown/release/near_atomic_swap.wasm")?;
        let contract = worker.dev_deploy(&wasm).await?;
        Ok(contract)
    }

    async fn fund(
        worker: &Worker<Sandbox>,
        contract: &Contract,
        caller: &Account,
        sender: &AccountId,
        receiver: &AccountId,
        amount: Balance,
        hashlock: [u8; 32],
        timelock: u64,
    ) -> Result<TransferId> {
        let res = caller
            .call(&worker, contract.id(), "fund")
            .args_json((sender, receiver, amount, hashlock, timelock))?
            .gas(300_000_000_000_000)
            .deposit(amount + 1)
            .transact()
            .await?;
        assert!(res.is_success());
        Ok(res.json()?)
    }

    async fn check(
        worker: &Worker<Sandbox>,
        contract: &Contract,
        caller: &Account,
        transfer_id: TransferId,
    ) -> Result<String> {
        let res = caller
            .call(&worker, contract.id(), "check")
            .args_json((transfer_id,))?
            .gas(300_000_000_000_000)
            .transact()
            .await?;
        assert!(res.is_success());
        Ok(res.json()?)
    }

    async fn confirm(
        worker: &Worker<Sandbox>,
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
        let account = worker.root_account()?;

        let sender = account
            .create_subaccount(&worker, "sender")
            .initial_balance(parse_near!("20 N"))
            .transact()
            .await?
            .into_result()?;

        let sender_balance = sender.view_account(&worker).await?.balance;
        assert_eq!(sender_balance, 20_000_000_000_000_000_000_000_000);

        let receiver = account
            .create_subaccount(&worker, "receiver")
            .initial_balance(parse_near!("2 N"))
            .transact()
            .await?
            .into_result()?;

        let receiver_balance = receiver.view_account(&worker).await?.balance;
        assert_eq!(receiver_balance, 2_000_000_000_000_000_000_000_000);

        let timelock = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let contract = init(&worker).await?;

        let transfer_id = fund(
            &worker,
            &contract,
            &sender,
            &sender.id(),
            &receiver.id(),
            10_000_000_000_000_000_000_000_000,
            [
                165, 152, 132, 76, 216, 153, 182, 114, 45, 89, 20, 251, 170, 95, 204, 77, 214, 166,
                43, 58, 171, 243, 206, 181, 109, 46, 63, 177, 197, 13, 234, 154,
            ],
            timelock,
        )
        .await?;

        let pending_event = check(&worker, &contract, &receiver, transfer_id).await?;
        // NOTE: the last field is timestamps
        assert!(pending_event.starts_with("Pending((AccountId(\"sender.test.near\"), AccountId(\"receiver.test.near\"), 10000000000000000000000000, "));

        confirm(
            &worker,
            &contract,
            &sender,
            &sender.id(),
            &receiver.id(),
            10_000_000_000_000_000_000_000_000,
            [
                165, 152, 132, 76, 216, 153, 182, 114, 45, 89, 20, 251, 170, 95, 204, 77, 214, 166,
                43, 58, 171, 243, 206, 181, 109, 46, 63, 177, 197, 13, 234, 154,
            ],
            timelock,
            *b"ssssssssssssssssssssssssssssssss",
        )
        .await?;

        let new_receiver_balance = receiver.view_account(&worker).await?.balance;

        assert!(
            new_receiver_balance > 11_990_000_000_000_000_000_000_000
                && new_receiver_balance < 12_000_000_000_000_000_000_000_000
        );

        let confirm_event = check(&worker, &contract, &receiver, transfer_id).await?;
        // NOTE: the last two fields are timestamps and secret
        assert!(confirm_event.starts_with("Confirmed((AccountId(\"sender.test.near\"), AccountId(\"receiver.test.near\"), 10000000000000000000000000, "));

        Ok(())
    }
}
