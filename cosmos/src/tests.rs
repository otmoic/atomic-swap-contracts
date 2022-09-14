#[cfg(test)]
mod tests {
    use std::time::SystemTime;

    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coin, from_binary, Deps, DepsMut};

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

    fn call_fund_with_correct_deposit(deps: DepsMut, msg: TransferMsg) {
        let info = mock_info(&msg.sender, &vec![msg.coin.clone()]);
        let _res =
            fund(deps, mock_env(), info, msg).expect("contract successfully handles FundMsg");
    }

    fn call_confirm_with_receiver(deps: DepsMut, msg: ConfirmMsg) {
        let info = mock_info(&msg.receiver, &vec![]);
        let _res =
            confirm(deps, mock_env(), info, msg).expect("contract successfully handles FundMsg");
    }

    #[test]
    fn call_fund_without_deposit() {
        let mut deps = mock_dependencies();

        let timelock = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let msg = TransferMsg {
            sender: "sender".into(),
            receiver: "receiver".into(),
            coin: coin(100, "atom"),
            hashlock: [
                165, 152, 132, 76, 216, 153, 182, 114, 45, 89, 20, 251, 170, 95, 204, 77, 214, 166,
                43, 58, 171, 243, 206, 181, 109, 46, 63, 177, 197, 13, 234, 154,
            ],
            timelock,
        };

        let info = mock_info(&msg.sender, &vec![]);
        assert_eq!(
            fund(deps.as_mut(), mock_env(), info, msg),
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

        let msg = TransferMsg {
            sender: "sender".into(),
            receiver: "receiver".into(),
            coin: coin(100, "atom"),
            hashlock: [
                165, 152, 132, 76, 216, 153, 182, 114, 45, 89, 20, 251, 170, 95, 204, 77, 214, 166,
                43, 58, 171, 243, 206, 181, 109, 46, 63, 177, 197, 13, 234, 154,
            ],
            timelock,
        };

        call_fund_with_correct_deposit(deps.as_mut(), msg.clone());
        assert_query(deps.as_ref(), msg.clone(), [0; 32], TransferStatus::Pending);

        let confirm_msg = ConfirmMsg {
            sender: "sender".into(),
            receiver: "receiver".into(),
            coin: coin(100, "atom"),
            hashlock: [
                165, 152, 132, 76, 216, 153, 182, 114, 45, 89, 20, 251, 170, 95, 204, 77, 214, 166,
                43, 58, 171, 243, 206, 181, 109, 46, 63, 177, 197, 13, 234, 154,
            ],
            timelock,
            secret: *b"ssssssssssssssssssssssssssssssss",
        };

        call_confirm_with_receiver(deps.as_mut(), confirm_msg);
        assert_query(
            deps.as_ref(),
            msg,
            *b"ssssssssssssssssssssssssssssssss",
            TransferStatus::Confirmed,
        );
    }
}
