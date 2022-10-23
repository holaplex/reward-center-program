use std::str::FromStr;

use anchor_lang::{prelude::Pubkey, AnchorDeserialize};
use anyhow::{Context, Result as AnyhowResult};
use hpl_reward_center::state::RewardCenter;
use log::info;
use solana_client::rpc_client::RpcClient;

/// # Errors
///
/// Will return `Err` if reward center address fails to parse
pub fn process_fetch_reward_center_state(
    client: &RpcClient,
    reward_center: &str,
) -> AnyhowResult<()> {
    let reward_center_pubkey = Pubkey::from_str(reward_center)
        .context("Failed to parse Pubkey from reward center string")?;

    let reward_center_data = client
        .get_account_data(&reward_center_pubkey)
        .context("Failed to get reward center data")?;

    let RewardCenter {
        auction_house,
        reward_rules,
        token_mint,
        ..
    } = RewardCenter::deserialize(&mut &reward_center_data[8..])?;

    info!("Reward Center address: {}", reward_center);
    info!("Auction house address: {}", auction_house.to_string());
    info!(
        "Reward Center rewards mint address: {}",
        token_mint.to_string()
    );
    info!(
        "Reward Center payout operation: {:?}",
        reward_rules.mathematical_operand
    );
    info!(
        "Reward Center reward payout basis points: {}",
        reward_rules.seller_reward_payout_basis_points
    );
    info!(
        "Reward Center payout numeral: {}",
        reward_rules.payout_numeral
    );

    Ok(())
}
