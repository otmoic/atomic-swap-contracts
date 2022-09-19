#[cfg(test)]
mod tests {
    use std::time::SystemTime;

    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coin, from_binary, Deps};

    use crate::*;

    fn assert_query(deps: Deps, msg: TransferMsg, secret_key: SecretKey, status: TransferStatus) {
        let res = query(deps, mock_env(), msg.clone()).unwrap();
        let record: TransferRecord = from_binary(&res).unwrap();
        assert_eq!(record.sender, msg.sender);
        assert_eq!(record.receiver, msg.receiver);
        assert_eq!(record.coin, msg.coin);
        assert_eq!(record.hashlock, msg.hashlock);
        assert_eq!(record.secret_key, secret_key);
        assert_eq!(record.status, status);
    }

    #[test]
    fn call_fund_without_deposit() {
        let mut deps = mock_dependencies();

        let timelock = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let msg = InstantiateMsg {
            platform: "platform".into(),
            fee: coin(1, "atom"),
        };

        let info = mock_info("sender", &vec![]);
        assert!(instantiate(deps.as_mut(), mock_env(), info, msg).is_ok());

        let msg = ExecuteMsg::Fund(TransferMsg {
            sender: "sender".into(),
            receiver: "receiver".into(),
            coin: coin(100, "atom"),
            hashlock: [
                165, 152, 132, 76, 216, 153, 182, 114, 45, 89, 20, 251, 170, 95, 204, 77, 214, 166,
                43, 58, 171, 243, 206, 181, 109, 46, 63, 177, 197, 13, 234, 154,
            ],
            timelock,
        });
        let info = mock_info("sender", &vec![]);

        assert_eq!(
            execute(deps.as_mut(), mock_env(), info, msg),
            Err(ContractError::InsufficientFundsSend)
        );
    }

    #[test]
    fn round_trip() {
        let mut deps = mock_dependencies();

        let timelock = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let msg = InstantiateMsg {
            platform: "platform".into(),
            fee: coin(1, "atom"),
        };

        let info = mock_info("sender", &vec![]);
        assert!(instantiate(deps.as_mut(), mock_env(), info, msg).is_ok());

        let msg = ExecuteMsg::Fund(TransferMsg {
            sender: "sender".into(),
            receiver: "receiver".into(),
            coin: coin(100, "atom"),
            hashlock: [
                165, 152, 132, 76, 216, 153, 182, 114, 45, 89, 20, 251, 170, 95, 204, 77, 214, 166,
                43, 58, 171, 243, 206, 181, 109, 46, 63, 177, 197, 13, 234, 154,
            ],
            timelock,
        });
        let info = mock_info("sender", &vec![coin(101, "atom")]);

        assert!(execute(deps.as_mut(), mock_env(), info, msg).is_ok());
        let transfer_msg = TransferMsg {
            sender: "sender".into(),
            receiver: "receiver".into(),
            coin: coin(100, "atom"),
            hashlock: [
                165, 152, 132, 76, 216, 153, 182, 114, 45, 89, 20, 251, 170, 95, 204, 77, 214, 166,
                43, 58, 171, 243, 206, 181, 109, 46, 63, 177, 197, 13, 234, 154,
            ],
            timelock,
        };

        assert_query(
            deps.as_ref(),
            transfer_msg.clone(),
            [0; 32],
            TransferStatus::Pending,
        );

        let msg = ExecuteMsg::Confirm((
            TransferMsg {
                sender: "sender".into(),
                receiver: "receiver".into(),
                coin: coin(100, "atom"),
                hashlock: [
                    165, 152, 132, 76, 216, 153, 182, 114, 45, 89, 20, 251, 170, 95, 204, 77, 214,
                    166, 43, 58, 171, 243, 206, 181, 109, 46, 63, 177, 197, 13, 234, 154,
                ],
                timelock,
            },
            *b"ssssssssssssssssssssssssssssssss",
        ));
        let info = mock_info("sender", &vec![]);

        assert!(execute(deps.as_mut(), mock_env(), info, msg).is_ok());

        assert_query(
            deps.as_ref(),
            transfer_msg,
            *b"ssssssssssssssssssssssssssssssss",
            TransferStatus::Confirmed,
        );
    }
}
