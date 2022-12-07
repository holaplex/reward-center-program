use std::str::FromStr;

use anchor_lang::{prelude::Pubkey, AnchorDeserialize};
use anyhow::{Context, Result as AnyhowResult};
use hpl_reward_center::state::RewardCenter;
use log::info;
use solana_client::rpc_client::RpcClient;
use spl_associated_token_account::get_associated_token_address;

/// # Errors
///
/// Will return `Err` if the following happens
/// 1. Reward center address fails to parse
/// 2. Reward center/rewards mint/reward center token account account does not exist
/// # Panics
///
/// Will panic if treasury balance amount does not parse
pub fn process_fetch_reward_center_treasury_balance(
    client: &RpcClient,
    reward_center: &str,
) -> AnyhowResult<()> {
    let reward_center_pubkey = Pubkey::from_str(reward_center)
        .context("Failed to parse Pubkey from reward center string")?;

    let reward_center_data = client
        .get_account_data(&reward_center_pubkey)
        .context("Failed to get reward center data")?;

    let RewardCenter { token_mint, .. } = RewardCenter::deserialize(&mut &reward_center_data[8..])?;

    info!("Token mint: {}", token_mint.to_string());

    let reward_center_rewards_token_account =
        get_associated_token_address(&reward_center_pubkey, &token_mint);

    let token_res = client
        .get_token_account_balance(&reward_center_rewards_token_account)
        .context("Unable to fetch reward center rewards balacne")?;

    let token_balance = token_res.ui_amount.unwrap_or_else(|| {
        (token_res.amount.parse::<f64>().unwrap()) / f64::from(token_res.decimals)
    });

    info!(
        "Reward center rewards mint address: {}",
        token_mint.to_string()
    );

    info!(
        "Reward center token account: {}",
        reward_center_rewards_token_account.to_string()
    );

    info!("Reward center treasury balance: {}", token_balance);

    Ok(())
}
