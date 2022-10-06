fn main() {}

#[cfg(test)]
mod test {
    use anyhow::Result;
    use near_units::parse_near;
    use std::time::SystemTime;
    use workspaces::{types::Balance, Account, AccountId, Contract, DevNetwork, Worker};

    async fn init(worker: &Worker<impl DevNetwork>) -> Result<Contract> {
        let wasm = std::fs::read("./target/wasm32-unknown-unknown/release/near_atomic_swap.wasm")?;
        let contract = worker.dev_deploy(&wasm).await?;
        Ok(contract)
    }

    async fn fund(
        contract: &Contract,
        caller: &Account,
        sender: &AccountId,
        receiver: &AccountId,
        amount: Balance,
        hashlock: [u8; 32],
        timelock: u64,
    ) -> Result<()> {
        let res = caller
            .call(contract.id(), "fund")
            .args_json((sender, receiver, amount, hashlock, timelock))
            .gas(300_000_000_000_000)
            .deposit(amount + 1)
            .transact()
            .await?;
        assert!(res.is_success());

        Ok(())
    }

    async fn refund(
        contract: &Contract,
        caller: &Account,
        sender: &AccountId,
        receiver: &AccountId,
        amount: Balance,
        hashlock: [u8; 32],
        timelock: u64,
    ) -> Result<()> {
        let res = caller
            .call(contract.id(), "refund")
            .args_json((sender, receiver, amount, hashlock, timelock))
            .gas(300_000_000_000_000)
            .transact()
            .await?;
        assert!(res.is_success());

        Ok(())
    }

    #[test_with::file("target/wasm32-unknown-unknown/release/near_atomic_swap.wasm")]
    #[tokio::test]
    async fn test_refund() -> Result<()> {
        let worker = workspaces::sandbox().await?;
        let account = worker.root_account()?;

        let sender = account
            .create_subaccount("sender")
            .initial_balance(parse_near!("20 N"))
            .transact()
            .await?
            .into_result()?;

        let mut sender_balance = sender.view_account().await?.balance;
        assert_eq!(sender_balance, 20_000_000_000_000_000_000_000_000);

        let receiver = account
            .create_subaccount("receiver")
            .initial_balance(parse_near!("2 N"))
            .transact()
            .await?
            .into_result()?;

        let receiver_balance = receiver.view_account().await?.balance;
        assert_eq!(receiver_balance, 2_000_000_000_000_000_000_000_000);

        let timelock = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let contract = init(&worker).await?;

        fund(
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

        refund(
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

        sender_balance = sender.view_account().await?.balance;
        assert!(
            sender_balance < 20_000_000_000_000_000_000_000_000
                && sender_balance > 19_900_000_000_000_000_000_000_000
        );

        Ok(())
    }
}
