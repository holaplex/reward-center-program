use std::{
    fs::File,
    path::{Path, PathBuf},
    str::FromStr,
};

use anyhow::{anyhow, Result as AnyhowResult};
use hpl_reward_center::{
    reward_centers::edit::EditRewardCenterParams,
    state::{PayoutOperation, RewardRules},
};
use hpl_reward_center_sdk::edit_reward_center;
use log::{error, info};
use retry::{delay::Exponential, retry};
use solana_client::rpc_client::RpcClient;
use solana_program::pubkey::Pubkey;
use solana_sdk::{signer::Signer, transaction::Transaction};

use crate::config::{parse_keypair, parse_solana_config};

pub fn process_edit_reward_center(
    client: RpcClient,
    keypair_path: Option<PathBuf>,
    reward_center: String,
    auction_house: String,
    config_file: PathBuf,
) -> AnyhowResult<()> {
    let solana_options = parse_solana_config()?;

    let keypair = parse_keypair(&keypair_path, &solana_options)?;

    let reward_center_pubkey = match Pubkey::from_str(&reward_center) {
        Ok(pubkey) => pubkey,
        Err(_) => return Err(anyhow!("Failed to parse Pubkey from reward center string")),
    };

    let auction_house_pubkey = match Pubkey::from_str(&auction_house) {
        Ok(pubkey) => pubkey,
        Err(_) => return Err(anyhow!("Failed to parse Pubkey from auction house string")),
    };

    let edit_reward_center_params: EditRewardCenterParams = if Path::new(&config_file).exists() {
        let create_reward_center_config_file = File::open(config_file)?;
        let edit_reward_center_config: crate::schema::EditRewardCenterParams =
            serde_json::from_reader(create_reward_center_config_file)?;

        EditRewardCenterParams {
            reward_rules: RewardRules {
                seller_reward_payout_basis_points: edit_reward_center_config
                    .seller_reward_payout_basis_points,
                mathematical_operand: match edit_reward_center_config.mathematical_operand {
                    crate::schema::PayoutOperation::Divide => PayoutOperation::Divide,
                    crate::schema::PayoutOperation::Multiple => PayoutOperation::Multiple,
                },
                payout_numeral: edit_reward_center_config.payout_numeral,
            },
        }
    } else {
        error!("Update reward center config doesn't exist");
        return Err(anyhow!("Update config missing"));
    };

    let edit_reward_center_ix = edit_reward_center(
        keypair.pubkey(),
        auction_house_pubkey,
        edit_reward_center_params,
    );

    info!(
        "Updating reward center {}",
        reward_center_pubkey.to_string()
    );

    let latest_blockhash = client.get_latest_blockhash()?;

    let transaction = Transaction::new_signed_with_payer(
        &[edit_reward_center_ix],
        Some(&keypair.pubkey()),
        &[&keypair],
        latest_blockhash,
    );

    let tx_hash = retry(
        Exponential::from_millis_with_factor(250, 2.0).take(3),
        || client.send_and_confirm_transaction(&transaction),
    )?;

    info!("Updated reward center in tx: {:?}", &tx_hash);

    Ok(())
}
