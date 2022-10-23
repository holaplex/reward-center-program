use std::str::FromStr;

use anchor_lang::{prelude::Pubkey, AnchorDeserialize};
use anyhow::{Context, Result as AnyhowResult};
use hpl_reward_center::state::RewardCenter;
use log::info;
use solana_client::rpc_client::RpcClient;
use solana_program::program_pack::Pack;
use spl_associated_token_account::get_associated_token_address;
use spl_token::state::{Account, Mint};

/// # Errors
///
/// Will return `Err` if the following happens
/// 1. Reward center address fails to parse
/// 2. Reward center/rewards mint/reward center token account account does not exist
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

    let token_mint_data = client
        .get_account_data(&token_mint)
        .context("Reward center rewards token mint does not exit")?;

    let Mint { decimals, .. } = Mint::unpack(&token_mint_data[..])?;

    let reward_center_rewards_token_account =
        get_associated_token_address(&reward_center_pubkey, &token_mint);

    let reward_center_rewards_token_account_data = client
        .get_account_data(&token_mint)
        .context("Reward center rewards token account does not exit")?;

    let Account { amount, .. } = Account::unpack(&reward_center_rewards_token_account_data[..])?;

    info!(
        "Reward center rewards mint address: {}",
        token_mint.to_string()
    );

    info!(
        "Reward center token account: {}",
        reward_center_rewards_token_account.to_string()
    );

    info!(
        "Reward center treasury balance: {}",
        amount.saturating_div(10u64.saturating_pow(decimals.into()))
    );

    Ok(())
}
