use anyhow::Result;
use solana_client::rpc_client::RpcClient;

pub fn process_create_reward_center(
    client: RpcClient,
    config_file: String,
    auction_house: Option<String>,
    rewards_mint: Option<String>,
) -> Result<()> {
    Ok(())
}
