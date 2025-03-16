use anyhow::{Context, Result};
use futures::future::join_all;
use serde::Deserialize;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use tokio;


const SOL_PER_LAMPORTS: u64 = 1_000_000_000;

#[derive(Debug, Deserialize)]
struct Config {
    rpc_url: String,
    wallets: Vec<String>,
}

fn get_wallets_from_strings(wallets: &Vec<String>) -> Result<Vec<Pubkey>> {
    let mut validated_wallets = Vec::new();
    for wallet_string in wallets {
        let pubkey = Pubkey::from_str(wallet_string)?;
        validated_wallets.push(pubkey);
    }
    Ok(validated_wallets)
}

async fn get_balance(client: &RpcClient, wallet: &Pubkey) -> Result<(String, u64)> {
    let balance = client.get_balance(wallet).await?;
    Ok((wallet.to_string(), balance))
}

async fn get_all_balances(client: &RpcClient, wallets: &Vec<Pubkey>) -> Result<Vec<(String, u64)>> {
    let futures: Vec<_> = wallets
        .iter()
        .map(|wallet| get_balance(client, wallet))
        .collect();

    let results = join_all(futures).await;
    let mut balances = Vec::new();

    for result in results {
        balances.push(result?);
    }

    Ok(balances)
}

async fn get_all_balances_2(
    client: &RpcClient,
    wallets: &Vec<Pubkey>,
) -> Result<Vec<(String, u64)>> {
    let accounts = client
        .get_multiple_accounts(wallets)
        .await?;

    Ok(accounts
        .into_iter()
        .enumerate()
        .map(|(i, acc)| (wallets[i].to_string(), acc.map(|a| a.lamports).unwrap_or(0)))
        .collect())
}

#[tokio::main]
async fn main() -> Result<()> {
    let config_content = std::fs::read_to_string("config.yaml")?;
    let config: Config = serde_yaml::from_str(&config_content)?;
    let client = RpcClient::new(config.rpc_url);

    let wallets = get_wallets_from_strings(&config.wallets)?;
    let start = std::time::Instant::now();
    let balances1 = get_all_balances(&client, &wallets).await?;
    let duration1 = start.elapsed();
    println!("Method 1 took: {:?}", duration1);

    let start = std::time::Instant::now();
    let balances2 = get_all_balances_2(&client, &wallets).await?;
    let duration2 = start.elapsed();
    println!("Method 2 took: {:?}", duration2);

    println!("\nResults from method 1:");
    for (wallet, balance) in balances1 {
        println!("Wallet: {}, Balance: {} SOL", wallet, (balance as f64) / SOL_PER_LAMPORTS as f64);
    }

    println!("\nResults from method 2:");
    for (wallet, balance) in balances2 {
        println!("Wallet: {}, Balance: {} SOL", wallet, (balance as f64) / SOL_PER_LAMPORTS as f64);
    }

    Ok(())
}
