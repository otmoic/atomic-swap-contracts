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
        let platform_key = Pubkey::new_from_array([
            248, 168, 61, 18, 213, 218, 160, 220, 199, 48, 254, 164, 209, 214, 235, 60, 128, 101,
            144, 242, 95, 58, 210, 60, 85, 146, 228, 120, 192, 220, 18, 161,
        ]);
        let sender_key = Pubkey::new_from_array(
            TryInto::<[u8; 32]>::try_into(
                bs58::decode("4MGCWdb7dyCiar6p6RLtmGUGioqzGcPSpzAy4pwdje84")
                    .into_vec()
                    .unwrap(),
            )
            .unwrap(),
        );
        let receiver_key = Pubkey::new_from_array(
            TryInto::<[u8; 32]>::try_into(
                bs58::decode("Dxd5TVxwTAx64VSLbhw96oMv25nLgvXVekhmdoF733VV")
                    .into_vec()
                    .unwrap(),
            )
            .unwrap(),
        );
        let contract_key = Pubkey::new_from_array([255; 32]);

        let mut program_test = ProgramTest::new("atomicswap", program_id, processor!(atomic_swap));
        // TODO: Adding system program transfer for the complete environment
        // program_test.add_builtin_program("transfer", system_program::id(), |_size, _ctx| -> { return Ok(()) } );

        let data = Storage::default().try_to_vec().unwrap();
        program_test.add_account(
            contract_key,
            Account {
                lamports: 5,
                data,
                owner: program_id,
                ..Account::default()
            },
        );
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
                data: Vec::new(),
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
            .get_account(contract_key)
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
                    AccountMeta::new(contract_key, false),
                    AccountMeta::new(payer.pubkey(), false),
                    AccountMeta::new(receiver_key, false),
                    AccountMeta::new(platform_key, false),
                ],
            )],
            Some(&payer.pubkey()),
        );
        transaction.sign(&[&payer], recent_blockhash);
        banks_client.process_transaction(transaction).await.unwrap();

        let contract_address = banks_client
            .get_account(contract_key)
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
                    AccountMeta::new(contract_key, false),
                    AccountMeta::new(sender_key, false),
                    AccountMeta::new(receiver_key, false),
                ],
            )],
            Some(&payer.pubkey()),
        );
        transaction.sign(&[&payer], recent_blockhash);
        banks_client.process_transaction(transaction).await.unwrap();

        let contract_address = banks_client
            .get_account(contract_key)
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
