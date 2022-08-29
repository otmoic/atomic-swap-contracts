use std::error::Error;
use std::io::{self, Write};

use solana_rpc_client::rpc_client::RpcClient;
use solana_sdk::signature::Signer;
use solana_sdk::signer::keypair::{read_keypair_file, Keypair};
use solana_streamer::socket::SocketAddrSpace;
use solana_test_validator::TestValidator;

fn main() -> Result<(), Box<dyn Error>> {
    let alice = ask_key_file("Enter Alice key file:")?;
    let bob = ask_key_file("Enter Bob key file:")?;

    let test_validator =
        TestValidator::with_no_fees(alice.pubkey(), None, SocketAddrSpace::Unspecified);

    let client = RpcClient::new(test_validator.rpc_url());

    let alice_balance = client.get_balance(&alice.pubkey());
    let bob_balance = client.get_balance(&bob.pubkey());
    println!("Alice balance: {alice_balance:?}");
    println!("Bob balance: {bob_balance:?}");

    Ok(())
}

fn ask_key_file(query: &str) -> Result<Keypair, Box<dyn Error>> {
    print!("{}", query);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    read_keypair_file(input.trim())
}
