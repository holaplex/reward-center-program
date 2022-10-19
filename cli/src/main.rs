use std::{str::FromStr, time::Duration};

use anyhow::Result;
use log::*;
use reward_center::{
    commands::{
        process_create_reward_center, process_edit_reward_center, process_fund_reward_center,
    },
    config::*,
    constants::*,
    opt::*,
};
use solana_client::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use structopt::StructOpt;

fn main() -> Result<()> {
    let Opt {
        log_level,
        rpc,
        timeout,
        cmd,
    } = Opt::from_args();

    solana_logger::setup_with(&log_level);

    let sol_config = parse_solana_config();

    let (rpc_url, commitment) = if let Some(cli_rpc_url) = rpc {
        (cli_rpc_url, String::from("confirmed"))
    } else if let Some(sol_config) = sol_config {
        (sol_config.json_rpc_url, sol_config.commitment)
    } else {
        info!(
            "Could not find a valid Solana-CLI config file. Defaulting to https://api.devnet.solana.com devnet node."
        );
        (
            String::from("https://api.devnet.solana.com"),
            String::from("confirmed"),
        )
    };

    if PUBLIC_RPC_URLS.contains(&rpc_url.as_str()) {
        warn!(
            "Using a public RPC URL is not recommended for heavy tasks as you will be rate-limited and suffer a performance hit"
        );
        warn!("Please use a private RPC endpoint for best performance results.");
        *USE_RATE_LIMIT.write().unwrap() = true;
    } else if RATE_LIMIT_DELAYS.contains_key(&rpc_url.as_str()) {
        *USE_RATE_LIMIT.write().unwrap() = true;
        *RPC_DELAY_NS.write().unwrap() = RATE_LIMIT_DELAYS[&rpc_url.as_str()];
    }

    let commitment_config = CommitmentConfig::from_str(&commitment)?;
    let timeout = Duration::from_secs(timeout);
    let confirm_transaction_initial_timeout = Duration::from_secs(60);

    let client = RpcClient::new_with_timeouts_and_commitment(
        rpc_url,
        timeout,
        commitment_config,
        confirm_transaction_initial_timeout,
    );

    match cmd {
        Command::Create {
            keypair,
            config_file,
            auction_house,
            mint_rewards,
        } => {
            process_create_reward_center(client, keypair, config_file, auction_house, mint_rewards)?
        },

        Command::Edit {
            keypair,
            config_file,
            reward_center,
            auction_house,
        } => {
            process_edit_reward_center(client, keypair, reward_center, auction_house, config_file)?
        },

        Command::Fund {
            reward_center,
            keypair,
            amount,
        } => process_fund_reward_center(client, keypair, reward_center, amount)?,
    }

    println!("Done :)");
    Ok(())
}
