use std::{path::PathBuf, str::FromStr};

use anchor_lang::AnchorDeserialize;
use anyhow::{anyhow, bail, Context, Result as AnyhowResult};
use hpl_reward_center::state::RewardCenter;
use log::{error, info};
use retry::{delay::Exponential, retry};
use solana_client::{client_error::ClientErrorKind, rpc_client::RpcClient, rpc_request::RpcError};
use solana_program::{
    instruction::Instruction, program_option::COption, program_pack::Pack, pubkey::Pubkey,
};
use solana_sdk::{signer::Signer, transaction::Transaction};
use spl_associated_token_account::get_associated_token_address;
use spl_token::{
    instruction::{mint_to_checked, transfer},
    state::{Account, Mint},
};

use crate::config::{parse_keypair, parse_solana_configuration};

/// # Errors
///
/// Will return `Err` if the following happens
/// 1. Reward center address fails to parse
/// 2. Reward center/rewards mint/reward center token account account does not exist
pub fn process_fund_reward_center(
    client: &RpcClient,
    keypair_path: &Option<PathBuf>,
    reward_center: &str,
    amount: u64,
) -> AnyhowResult<()> {
    let solana_options = parse_solana_configuration()?;

    let keypair = parse_keypair(keypair_path, &solana_options)?;
    let token_program = spl_token::id();

    let reward_center_pubkey = Pubkey::from_str(reward_center)
        .context("Failed to parse Pubkey from mint rewards string")?;

    info!("Getting reward center data");
    let reward_center_data = client.get_account_data(&reward_center_pubkey)?;
    let RewardCenter { token_mint, .. } = RewardCenter::deserialize(&mut &reward_center_data[8..])?;

    info!("Getting token mint data");
    let token_mint_data = client.get_account_data(&token_mint)?;

    let Mint {
        mint_authority,
        decimals,
        ..
    } = Mint::unpack(&token_mint_data[..])?;

    let caller_reward_mint_token_account =
        get_associated_token_address(&keypair.pubkey(), &token_mint);

    let reward_center_reward_mint_token_account =
        get_associated_token_address(&reward_center_pubkey, &token_mint);

    let amount_to_transfer_with_decimals =
        amount.saturating_mul(10u64.saturating_pow(decimals.into()));

    let instructions: Vec<Instruction> =
        match client.get_account_data(&caller_reward_mint_token_account) {
            Ok(data) => {
                let Account {
                    amount: token_balance,
                    ..
                } = Account::unpack(&data[..])?;

                if token_balance < amount_to_transfer_with_decimals {
                    if let COption::Some(mint_authority) = mint_authority {
                        if mint_authority.eq(&keypair.pubkey()) {
                            vec![mint_to_checked(
                                &token_program,
                                &token_mint,
                                &reward_center_reward_mint_token_account,
                                &keypair.pubkey(),
                                &[],
                                amount_to_transfer_with_decimals,
                                decimals,
                            )?]
                        } else {
                            error!(
                            "Caller reward token account does not have enough tokens to transfer"
                        );
                            return Err(anyhow!(
                            "Caller reward token account does not have enough tokens to transfer"
                        ));
                        }
                    } else {
                        error!("Mint authority parse failed");
                        return Err(anyhow!("Error in mint authority account parse"));
                    }
                } else {
                    vec![transfer(
                        &token_program,
                        &caller_reward_mint_token_account,
                        &reward_center_reward_mint_token_account,
                        &keypair.pubkey(),
                        &[&keypair.pubkey()],
                        amount_to_transfer_with_decimals,
                    )?]
                }
            },
            Err(err) if matches!(err.kind(), ClientErrorKind::RpcError(RpcError::ForUser(_))) => {
                if let COption::Some(mint_authority) = mint_authority {
                    if mint_authority.eq(&keypair.pubkey()) {
                        vec![mint_to_checked(
                            &token_program,
                            &token_mint,
                            &reward_center_reward_mint_token_account,
                            &keypair.pubkey(),
                            &[],
                            amount_to_transfer_with_decimals,
                            decimals,
                        )?]
                    } else {
                        bail!("Caller reward token account does not exist");
                    }
                } else {
                    bail!("Error in mint authority account parse");
                }
            },
            Err(err) => return Err(err).context("Failed to get account data for rewards mint"),
        };

    let latest_blockhash = client.get_latest_blockhash()?;
    let transaction = Transaction::new_signed_with_payer(
        &instructions,
        Some(&keypair.pubkey()),
        &[&keypair],
        latest_blockhash,
    );

    info!("Funding {} tokens to reward center", amount);

    let tx_hash = retry(
        Exponential::from_millis_with_factor(250, 2.0).take(3),
        || client.send_and_confirm_transaction(&transaction),
    )?;

    info!("Funding complete. Tx hash {}", tx_hash);

    Ok(())
}
