fn main() {}

#[cfg(test)]
mod test {
    use std::time::{Duration, SystemTime};

    use atomicswap::{atomic_swap, Method, Storage, TransferStatus};
    use borsh::{BorshDeserialize, BorshSerialize};
    use solana_program_test::*;
    use solana_sdk::{
        account::Account,
        instruction::{AccountMeta, Instruction},
        pubkey::Pubkey,
        signature::Signer,
        transaction::Transaction,
    };

    #[tokio::test]
    async fn round_trip() {
        let program_id = Pubkey::new_unique();
        let platform_key = Pubkey::new_from_array([255; 32]);
        let sender_key = Pubkey::new_from_array([1; 32]);
        let receiver_key = Pubkey::new_from_array([2; 32]);

        let mut program_test = ProgramTest::new("atomicswap", program_id, processor!(atomic_swap));
        let data = Storage::default().try_to_vec().unwrap();
        program_test.add_account(
            platform_key,
            Account {
                lamports: 0,
                data: Vec::new(),
                owner: program_id,
                ..Account::default()
            },
        );
        program_test.add_account(
            sender_key,
            Account {
                lamports: 5,
                data,
                owner: program_id,
                ..Account::default()
            },
        );
        program_test.add_account(
            receiver_key,
            Account {
                lamports: 0,
                data: Vec::new(),
                owner: program_id,
                ..Account::default()
            },
        );

        let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

        let contract_address = banks_client
            .get_account(sender_key)
            .await
            .expect("get account should work")
            .expect("contract address not found");
        assert_eq!(
            Storage::try_from_slice(&contract_address.data)
                .unwrap()
                .status,
            TransferStatus::Initializd
        );

        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();
        let five_seconds_later = now + Duration::new(5, 0);

        let method = Method::Fund(
            100,
            [
                165, 152, 132, 76, 216, 153, 182, 114, 45, 89, 20, 251, 170, 95, 204, 77, 214, 166,
                43, 58, 171, 243, 206, 181, 109, 46, 63, 177, 197, 13, 234, 154,
            ],
            five_seconds_later.as_secs(),
        );

        let mut transaction = Transaction::new_with_payer(
            &[Instruction::new_with_borsh(
                program_id,
                &method,
                vec![
                    AccountMeta::new(sender_key, false),
                    AccountMeta::new(sender_key, false),
                    AccountMeta::new(receiver_key, false),
                    AccountMeta::new(platform_key, false),
                ],
            )],
            Some(&payer.pubkey()),
        );
        transaction.sign(&[&payer], recent_blockhash);
        banks_client.process_transaction(transaction).await.unwrap();

        let contract_address = banks_client
            .get_account(sender_key)
            .await
            .expect("get account should work")
            .expect("contract address not found");
        assert_eq!(
            Storage::try_from_slice(&contract_address.data)
                .unwrap()
                .status,
            TransferStatus::Pending
        );

        let method = Method::Confirm(
            100,
            [
                165, 152, 132, 76, 216, 153, 182, 114, 45, 89, 20, 251, 170, 95, 204, 77, 214, 166,
                43, 58, 171, 243, 206, 181, 109, 46, 63, 177, 197, 13, 234, 154,
            ],
            five_seconds_later.as_secs(),
            *b"ssssssssssssssssssssssssssssssss",
        );

        let mut transaction = Transaction::new_with_payer(
            &[Instruction::new_with_borsh(
                program_id,
                &method,
                vec![
                    AccountMeta::new(sender_key, false),
                    AccountMeta::new(sender_key, false),
                    AccountMeta::new(receiver_key, false),
                ],
            )],
            Some(&payer.pubkey()),
        );
        transaction.sign(&[&payer], recent_blockhash);
        banks_client.process_transaction(transaction).await.unwrap();

        let contract_address = banks_client
            .get_account(sender_key)
            .await
            .expect("get account should work")
            .expect("contract address not found");
        assert_eq!(
            Storage::try_from_slice(&contract_address.data)
                .unwrap()
                .status,
            TransferStatus::Confirmed
        );
    }
}
