use std::{path::PathBuf, str::FromStr, vec};

use crate::config::{parse_keypair, parse_solana_configuration};
use anchor_lang::{prelude::Pubkey, AnchorDeserialize};
use anyhow::{bail, Context, Result as AnyhowResult};
use hpl_reward_center::{pda::find_reward_center_address, state::RewardCenter};
use log::{error, info};
use mpl_auction_house::{
    pda::{find_auctioneer_pda, find_program_as_signer_address},
    AuctionHouse,
};
use solana_address_lookup_table_program::instruction::{create_lookup_table, extend_lookup_table};
use solana_client::rpc_client::RpcClient;
use solana_program::instruction::Instruction;
use solana_sdk::{commitment_config::CommitmentConfig, signer::Signer, transaction::Transaction};
use spl_associated_token_account::get_associated_token_address;

/// # Errors
///
/// Will return `Err` if the following happens
/// 1. Auction House/Keypair Path fails to parse/open
/// 2. Transaction errors due to validation
/// 3. RPC Errors if timed out
pub fn process_create_address_table_lookup(
    client: &RpcClient,
    keypair_path: &Option<PathBuf>,
    auction_house: &str,
) -> AnyhowResult<()> {
    let solana_options = parse_solana_configuration()?;

    let keypair = parse_keypair(keypair_path, &solana_options)?;

    let auction_house_pubkey = Pubkey::from_str(auction_house)
        .context("Failed to parse Pubkey from auction_house string")?;

    let auction_house_data = client
        .get_account_data(&auction_house_pubkey)
        .context("Failed to get auction house data")?;

    let AuctionHouse {
        authority: auction_house_authority,
        treasury_mint,
        auction_house_fee_account,
        auction_house_treasury,
        ..
    } = AuctionHouse::deserialize(&mut &auction_house_data[8..])?;

    let (reward_center_pubkey, _) = find_reward_center_address(&auction_house_pubkey);

    let reward_center_data = client
        .get_account_data(&reward_center_pubkey)
        .context("Failed to get reward center data")?;

    let RewardCenter { token_mint, .. } = RewardCenter::deserialize(&mut &reward_center_data[8..])?;

    if auction_house_authority.ne(&keypair.pubkey()) {
        error!("Given authority does not match with auction house authority");
        bail!("Auction authority address mismatch")
    }

    let reward_center_reward_token_account =
        get_associated_token_address(&reward_center_pubkey, &token_mint);

    let addresses = vec![
        auction_house_pubkey,
        find_auctioneer_pda(&auction_house_pubkey, &auction_house_authority).0,
        reward_center_pubkey,
        auction_house_treasury,
        auction_house_fee_account,
        auction_house_authority,
        spl_associated_token_account::id(),
        // token_account,
        treasury_mint,
        reward_center_reward_token_account,
        find_program_as_signer_address().0,
    ];

    let recent_slot = client
        .get_slot_with_commitment(CommitmentConfig::finalized())
        .context("Failed to fetch recent slot")?;

    let (create_address_lookup_table_ix, address_lookup_table_pubkey) =
        create_lookup_table(keypair.pubkey(), keypair.pubkey(), recent_slot);

    let extend_lookup_table_ix = extend_lookup_table(
        address_lookup_table_pubkey,
        keypair.pubkey(),
        Some(keypair.pubkey()),
        addresses,
    );

    let instructions: Vec<Instruction> =
        vec![create_address_lookup_table_ix, extend_lookup_table_ix];

    let latest_blockhash = client
        .get_latest_blockhash()
        .context("Failed to get latest blockhash")?;

    let tx_hash = client.send_and_confirm_transaction(&Transaction::new_signed_with_payer(
        &instructions,
        Some(&keypair.pubkey()),
        &[&keypair],
        latest_blockhash,
    ));

    match tx_hash {
        Ok(signature) => {
            info!("Created in tx: {:?}", &signature);
        },
        Err(error) => {
            error!("{:?}", error);
            bail!("Failed to send the transaction")
        },
    };

    info!(
        "Address table lookup created successfully. Address: {}",
        address_lookup_table_pubkey.to_string()
    );

    Ok(())
}
