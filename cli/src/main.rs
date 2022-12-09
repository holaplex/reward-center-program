#![deny(
    clippy::disallowed_methods,
    clippy::suspicious,
    clippy::style,
    missing_debug_implementations,
    missing_copy_implementations
)]
#![warn(clippy::pedantic)]

use std::{str::FromStr, time::Duration};

use anyhow::Result;
use clap::Parser;
use log::{error, info, warn};
use reward_center_cli::{
    commands::{
        process_create_address_table_lookup, process_create_reward_center,
        process_edit_reward_center, process_fetch_reward_center_state,
        process_fetch_reward_center_treasury_balance, process_fund_reward_center,
        process_withdraw_auction_house_treasury, process_withdraw_reward_center_treasury,
    },
    config::parse_solana_configuration,
    constants::PUBLIC_RPC_URLS,
    opt::{Command, Opt},
};
use solana_client::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .parse_default_env()
        .init();
    std::process::exit(match run() {
        Ok(()) => 0,
        Err(e) => {
            error!("{:?}", e);
            1
        },
    });
}

fn run() -> Result<()> {
    let Opt { rpc, timeout, cmd } = Opt::parse();

    let (rpc_url, commitment) = if let Some(cli_rpc_url) = rpc {
        (cli_rpc_url, "confirmed".into())
    } else if let Some(sol_config) = parse_solana_configuration()? {
        (sol_config.json_rpc_url, sol_config.commitment)
    } else {
        info!(
            "Could not find a valid Solana-CLI config file. Defaulting to https://api.devnet.solana.com devnet node."
        );
        ("https://api.devnet.solana.com".into(), "confirmed".into())
    };

    if PUBLIC_RPC_URLS.contains(&rpc_url.as_str()) {
        warn!(
            "Using a public RPC URL is not recommended for heavy tasks as you will be rate-limited and suffer a performance hit"
        );
        warn!("Please use a private RPC endpoint for best performance results.");
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
        } => process_create_reward_center(
            &client,
            &keypair,
            config_file,
            &auction_house,
            &mint_rewards,
        )?,

        Command::CreateAddressTable {
            auction_house,
            keypair,
        } => process_create_address_table_lookup(&client, &keypair, &auction_house)?,

        Command::Edit {
            keypair,
            config_file,
            reward_center,
            auction_house,
        } => process_edit_reward_center(
            &client,
            &keypair,
            &reward_center,
            &auction_house,
            config_file,
        )?,

        Command::Fund {
            reward_center,
            keypair,
            amount,
        } => process_fund_reward_center(&client, &keypair, &reward_center, amount)?,

        Command::FetchRewardCenterState { reward_center, .. } => {
            process_fetch_reward_center_state(&client, &reward_center)?;
        },
        Command::FetchTreasuryBalance { reward_center, .. } => {
            process_fetch_reward_center_treasury_balance(&client, &reward_center)?;
        },

        Command::WithdrawAuctionHouse {
            auction_house,
            keypair,
            amount,
        } => process_withdraw_auction_house_treasury(&client, &keypair, &auction_house, amount)?,

        Command::WithdrawRewardCenter {
            reward_center,
            keypair,
            amount,
        } => process_withdraw_reward_center_treasury(&client, &keypair, &reward_center, amount)?,
    }

    info!("Done :)");
    Ok(())
}
