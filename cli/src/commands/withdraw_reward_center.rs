use std::{path::PathBuf, str::FromStr};

use anchor_lang::AnchorDeserialize;
use anyhow::{anyhow, bail, Context, Result as AnyhowResult};
use hpl_reward_center::state::RewardCenter;
use hpl_reward_center_sdk::accounts::WithdrawRewardCenterFundsAccounts;
use hpl_reward_center_sdk::withdraw_reward_center_funds;
use log::{error, info};
use retry::{delay::Exponential, retry};
use solana_client::{client_error::ClientErrorKind, rpc_client::RpcClient, rpc_request::RpcError};
use solana_program::{instruction::Instruction, program_pack::Pack, pubkey::Pubkey};
use solana_sdk::{signer::Signer, transaction::Transaction};
use spl_associated_token_account::get_associated_token_address;
use spl_token::state::{Account, Mint};

use crate::config::{parse_keypair, parse_solana_configuration};

/// # Errors
///
/// Will return `Err` if the following happens
/// 1. Reward center address fails to parse
/// 2. Withdrawal amount is greater than the treasury balance
pub fn process_withdraw_reward_center_treasury(
    client: &RpcClient,
    keypair_path: &Option<PathBuf>,
    reward_center: &str,
    amount: u64,
) -> AnyhowResult<()> {
    let solana_options = parse_solana_configuration()?;

    let keypair = parse_keypair(keypair_path, &solana_options)?;

    let reward_center_pubkey = Pubkey::from_str(reward_center)
        .context("Failed to parse Pubkey from mint rewards string")?;

    info!("Getting reward center data");
    let reward_center_data = client.get_account_data(&reward_center_pubkey)?;
    let RewardCenter {
        token_mint,
        auction_house,
        ..
    } = RewardCenter::deserialize(&mut &reward_center_data[8..])?;

    info!("Getting rewards mint data");
    let token_mint_data = client.get_account_data(&token_mint)?;

    let Mint { decimals, .. } = Mint::unpack(&token_mint_data[..])?;

    let reward_center_reward_mint_token_account =
        get_associated_token_address(&reward_center_pubkey, &token_mint);

    let amount_to_withdraw_with_decimals =
        amount.saturating_mul(10u64.saturating_pow(decimals.into()));

    let instructions: Vec<Instruction> =
        match client.get_account_data(&reward_center_reward_mint_token_account) {
            Ok(data) => {
                let Account {
                    amount: token_balance,
                    ..
                } = Account::unpack(&data[..])?;

                if token_balance < amount_to_withdraw_with_decimals {
                    error!(
                    "Reward center reward token account does not have enough tokens to withdraw"
                );
                    return Err(anyhow!(
                    "Reward center reward token account does not have enough tokens to withdraw"
                ));
                }

                vec![withdraw_reward_center_funds(
                    WithdrawRewardCenterFundsAccounts {
                        wallet: keypair.pubkey(),
                        rewards_mint: token_mint,
                        auction_house,
                    },
                    amount_to_withdraw_with_decimals,
                )]
            },
            Err(err) if matches!(err.kind(), ClientErrorKind::RpcError(RpcError::ForUser(_))) => {
                bail!("Reward center reward token account does not exist");
            },
            Err(err) => {
                return Err(err).context("Failed to get reward center reward token account data")
            },
        };

    let latest_blockhash = client.get_latest_blockhash()?;
    let transaction = Transaction::new_signed_with_payer(
        &instructions,
        Some(&keypair.pubkey()),
        &[&keypair],
        latest_blockhash,
    );

    info!("Withdrawing {} tokens from reward center", amount);

    let tx_hash = retry(
        Exponential::from_millis_with_factor(250, 2.0).take(3),
        || client.send_and_confirm_transaction(&transaction),
    )?;

    info!("Withdrawal complete. Tx hash {}", tx_hash);

    Ok(())
}
