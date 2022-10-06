/// This is a HTLC scenario with detail document
///
/// Alice and Bob atomic exchange 10 near token to 50 near token with HTLC
///
fn main() {}

#[cfg(test)]
mod test {
    use near_atomic_swap::TransferId;

    use anyhow::Result;
    use near_units::parse_near;
    use std::time::{Duration, SystemTime};
    use workspaces::{network::Sandbox, types::Balance, Account, AccountId, Contract, Worker};

    async fn init(worker: &Worker<Sandbox>) -> Result<Contract> {
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
    ) -> Result<TransferId> {
        let res = caller
            .call(contract.id(), "fund")
            .args_json((sender, receiver, amount, hashlock, timelock))
            .gas(300_000_000_000_000)
            .deposit(amount + 1)
            .transact()
            .await?;
        assert!(res.is_success());
        Ok(res.json()?)
    }

    async fn check(
        contract: &Contract,
        caller: &Account,
        transfer_id: TransferId,
    ) -> Result<String> {
        let res = caller
            .call(contract.id(), "check")
            .args_json((transfer_id,))
            .gas(300_000_000_000_000)
            .transact()
            .await?;
        assert!(res.is_success());
        Ok(res.json()?)
    }

    async fn confirm(
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
            .call(contract.id(), "confirm")
            .args_json((sender, receiver, amount, hashlock, timelock, secret_key))
            .gas(300_000_000_000_000)
            .transact()
            .await?;
        assert!(res.is_success());

        Ok(())
    }

    /// In this scenario,
    /// Alice and Bob atomic exchange 10 near token to 50 near token
    #[test_with::file("target/wasm32-unknown-unknown/release/near_atomic_swap.wasm")]
    #[tokio::test]
    async fn two_roles_round_trip() -> Result<()> {
        let worker = workspaces::sandbox().await?;
        let account = worker.root_account()?;

        let alice = account
            .create_subaccount("alice")
            .initial_balance(parse_near!("100 N"))
            .transact()
            .await?
            .into_result()?;

        let alice_balance = alice.view_account().await?.balance;
        assert_eq!(alice_balance, 100_000_000_000_000_000_000_000_000);

        let bob = account
            .create_subaccount("bob")
            .initial_balance(parse_near!("100 N"))
            .transact()
            .await?
            .into_result()?;

        let bob_balance = bob.view_account().await?.balance;
        assert_eq!(bob_balance, 100_000_000_000_000_000_000_000_000);

        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();

        let twenty_mins_lock = (now + Duration::new(1200, 0)).as_secs();
        let contract = init(&worker).await?;

        // Alice make a HTLC to transfer 20 Near from Alice to Bob, with 10 mins lock
        // She can use `utils::gen_lock` to preset a pair of hashlock and secret,
        // and she knows the secret key to the hashlock now, she will not share the secret to Bob
        //
        // After fund called, Alice can get the transfer_id from the return.
        // Then she passes the transfer_id to Bob by other channel
        let transfer_id = fund(
            &contract,
            &alice,
            &alice.id(),
            &bob.id(),
            10_000_000_000_000_000_000_000_000,
            [
                165, 152, 132, 76, 216, 153, 182, 114, 45, 89, 20, 251, 170, 95, 204, 77, 214, 166,
                43, 58, 171, 243, 206, 181, 109, 46, 63, 177, 197, 13, 234, 154,
            ],
            twenty_mins_lock,
        )
        .await?;

        // When Bob having the transfer_id, he can check the transfer is there and the details are as the same as Alice say or not?
        // And the meanwhile Bob can know the hashlock
        // The hashlock is [165, 152, 132, 76, 216, 153, 182, 114, 45, 89, 20, 251, 170, 95, 204, 77, 214, 166, 43, 58, 171, 243, 206, 181, 109, 46, 63, 177, 197, 13, 234, 154]
        let pending_event = check(&contract, &bob, transfer_id).await?;

        // Note: the last two fields are timestamps and hashlock
        assert!(pending_event.starts_with("Pending((AccountId(\"alice.test.near\"), AccountId(\"bob.test.near\"), 10000000000000000000000000, "));
        assert!(pending_event.ends_with("[165, 152, 132, 76, 216, 153, 182, 114, 45, 89, 20, 251, 170, 95, 204, 77, 214, 166, 43, 58, 171, 243, 206, 181, 109, 46, 63, 177, 197, 13, 234, 154]))"));

        // Bob can check Alice's HTLC(transfer 1) is pending
        // ```
        // Pending(
        //      (
        //          AccountId("alice.test.near"),
        //          AccountId("bob.test.near"),
        //          10000000000000000000000000,
        //          ..., // the timestamp of twenty minis later
        //          [165, 152, 132, 76, 216, 153, 182, 114, 45, 89, 20, 251, 170, 95, 204, 77, 214, 166, 43, 58, 171, 243, 206, 181, 109, 46, 63, 177, 197, 13, 234, 154]
        //      )
        // )
        //
        // ```

        // After checking the HTLC(transfer 1) from Alice,
        // Bob can reuse the hashLock to create another HTLC(transfer 2) without knowing the secret key to the hashlock,
        // This new transfer is to transfer 5 Near from Bob to Alice with 10 mins timelock.
        //
        // After the fund called, Bob will get transfer_id2.
        // Also, he passes the transfer_id2 to Alic by other channel
        let ten_mins_lock = (now + Duration::new(600, 0)).as_secs();
        let transfer_id2 = fund(
            &contract,
            &bob,
            &bob.id(),
            &alice.id(),
            5_000_000_000_000_000_000_000_000,
            [
                165, 152, 132, 76, 216, 153, 182, 114, 45, 89, 20, 251, 170, 95, 204, 77, 214, 166,
                43, 58, 171, 243, 206, 181, 109, 46, 63, 177, 197, 13, 234, 154,
            ],
            ten_mins_lock,
        )
        .await?;

        // When Alice having the transfer_id2, she can check the transfer is there, and the transfer details are as the same as Bob say or not?
        let pending_event = check(&contract, &alice, transfer_id2).await?;

        // Note: the last two fields are timestamps and hashlock
        assert!(pending_event.starts_with("Pending((AccountId(\"bob.test.near\"), AccountId(\"alice.test.near\"), 5000000000000000000000000, "));
        assert!(pending_event.ends_with("[165, 152, 132, 76, 216, 153, 182, 114, 45, 89, 20, 251, 170, 95, 204, 77, 214, 166, 43, 58, 171, 243, 206, 181, 109, 46, 63, 177, 197, 13, 234, 154]))"));

        // Alice can check Bob's HTLC(transfer 2) is pending, and it will transfer 5 Near from Bob to Alice after confirm
        // ```
        // Pending(
        //      (
        //          AccountId("bob.test.near"),
        //          AccountId("alice.test.near"),
        //          5000000000000000000000000,
        //          ..., // the timestamp of ten minis later
        //          [165, 152, 132, 76, 216, 153, 182, 114, 45, 89, 20, 251, 170, 95, 204, 77, 214, 166, 43, 58, 171, 243, 206, 181, 109, 46, 63, 177, 197, 13, 234, 154]
        //      )
        // )
        //
        // ```

        // Now Alice can use the secret key to confirm Bob's HTLC(transfer 2) and get the money (5 Near)
        // The secret is *b"ssssssssssssssssssssssssssssssss"
        confirm(
            &contract,
            &alice,
            &bob.id(),
            &alice.id(),
            5_000_000_000_000_000_000_000_000,
            [
                165, 152, 132, 76, 216, 153, 182, 114, 45, 89, 20, 251, 170, 95, 204, 77, 214, 166,
                43, 58, 171, 243, 206, 181, 109, 46, 63, 177, 197, 13, 234, 154,
            ],
            ten_mins_lock,
            *b"ssssssssssssssssssssssssssssssss",
        )
        .await?;

        // After Alice confirms the contract, Bob can check his HTLC(transfer 2) and know the secret without asking Alice
        let confirm_event = check(&contract, &bob, transfer_id2).await?;
        // NOTE: the last two fields are timestamps and secret
        assert!(confirm_event.starts_with("Confirmed((AccountId(\"bob.test.near\"), AccountId(\"alice.test.near\"), 5000000000000000000000000, "));
        assert!(confirm_event.ends_with("[115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115]))"));
        // Bob can check his HTLC(transfer 2) is confirmed, and see the secret is
        // "[115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115]
        // ```
        // Confirmed(
        //      (
        //          AccountId("bob.test.near"),
        //          AccountId("alice.test.near"),
        //          5000000000000000000000000,
        //          ...,
        //          "[115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115]
        //       )
        // )
        // ```

        // Now Bob can use the same secret key to confirm Alice's HTLC(transfer 1) and get the money ( 10 Near )
        // The secret is "[115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115]
        confirm(
            &contract,
            &bob,
            &alice.id(),
            &bob.id(),
            10_000_000_000_000_000_000_000_000,
            [
                165, 152, 132, 76, 216, 153, 182, 114, 45, 89, 20, 251, 170, 95, 204, 77, 214, 166,
                43, 58, 171, 243, 206, 181, 109, 46, 63, 177, 197, 13, 234, 154,
            ],
            twenty_mins_lock,
            [
                115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115,
                115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115,
            ],
        )
        .await?;

        // Overall, Alice has 95 Near (100 - 10 + 5) and Bob has 105 Near ( 100 + 10 - 5 )
        // Note: there are some fee here
        let new_alice_balance = alice.view_account().await?.balance;
        assert!(
            new_alice_balance > 94_900_000_000_000_000_000_000_000
                && new_alice_balance < 95_000_000_000_000_000_000_000_000
        );

        let new_bob_balance = bob.view_account().await?.balance;
        assert!(
            new_bob_balance > 104_900_000_000_000_000_000_000_000
                && new_bob_balance < 105_000_000_000_000_000_000_000_000
        );

        Ok(())
    }
}
