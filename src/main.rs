use std::error::Error;
use std::str::FromStr;
use alloy::primitives::{hex, Address};
use alloy::providers::{Provider, ProviderBuilder};
use alloy::primitives::utils::format_ether;
use k256::ecdsa::{SigningKey, VerifyingKey};
use k256::elliptic_curve::rand_core::OsRng;
use k256::elliptic_curve::sec1::ToEncodedPoint;
use sha3::{Digest, Keccak256};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "Ethereum CLI Wallet", about = "A CLI Ethereum wallet in Rust")]
struct Cli {
    #[structopt(subcommand)]
    command: Command,
}

#[derive(StructOpt, Debug)]
enum Command {
    CreateWallet,
    Balance { address: String },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::from_args();

    match args.command {
        Command::CreateWallet => {
            create_wallet();
        }
        Command::Balance { address } => {
            get_balance(address).await?;
        }
    }

    Ok(())
}

// Function to generate an Ethereum wallet keypair
fn create_wallet() {
    // Generate a random private key using k256
    let signing_key = SigningKey::random(&mut OsRng);

    // Derive the public key from the private key
    let verifying_key = VerifyingKey::from(&signing_key);

    // Get the public key bytes and apply Keccak-256 to compute the Ethereum address
    let pubkey_uncompressed = verifying_key.to_encoded_point(false); // false = uncompressed format
    let pubkey_bytes = pubkey_uncompressed.to_bytes();

    // Compute the Ethereum address (last 20 bytes of the Keccak-256 hash of the public key)
    let mut hasher = Keccak256::new();
    hasher.update(&pubkey_bytes[1..]); // Skip the 0x04 prefix
    let result = hasher.finalize();
    let address_bytes = &result[12..]; // Ethereum address is the last 20 bytes    

    // Convert the address to alloy's Address type
    let address: Address = Address::from_slice(address_bytes);

    println!("Your new wallet address: {}", address);
    println!("Your private key (keep it safe!): {}", hex::encode(signing_key.to_bytes()));
}

async fn get_balance(address: String) -> Result<(), Box<dyn Error>> {
    // Set up the HTTP transport which is consumed by the RPC client.
    let rpc_url = "https://eth.merkle.io".parse()?;

    // Create a provider with the HTTP transport using the `reqwest` crate.
    let provider = ProviderBuilder::new().on_http(rpc_url);

    // Create an Address instance from the address String
    let address = Address::from_str(&address)?;

    // Fetch the current balance of the address in wei and convert it to ether
    let balance_in_wei = provider.get_balance(address).await?;
    let balance_in_ether = format_ether(balance_in_wei);

    // Print the results
    println!("Balance for address {} is {} ETH", address, balance_in_ether);

    Ok(())
}
