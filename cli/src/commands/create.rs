use std::{
    fs::File,
    path::{Path, PathBuf},
    str::FromStr,
};

use anyhow::{bail, Context, Result as AnyhowResult};
use hpl_reward_center::pda::find_reward_center_address;
use hpl_reward_center_sdk::accounts::CreateRewardCenterAccounts;
use hpl_reward_center_sdk::create_reward_center;
use log::{error, info, warn};
use mpl_auction_house::{
    pda::{
        find_auction_house_address, find_auction_house_fee_account_address,
        find_auction_house_treasury_address,
    },
    state::AuthorityScope,
};
use mpl_auction_house_sdk::{
    accounts::{CreateAuctionHouseAccounts, DelegateAuctioneerAccounts},
    args::{CreateAuctionHouseData, DelegateAuctioneerData},
    create_auction_house, delegate_auctioneer,
};
use solana_client::rpc_client::RpcClient;
use solana_program::{
    instruction::Instruction, program_pack::Pack, pubkey::Pubkey, system_instruction::transfer,
};
use solana_sdk::{
    signature::Keypair, signer::Signer, system_instruction::create_account,
    transaction::Transaction,
};
use spl_associated_token_account::{
    get_associated_token_address, instruction::create_associated_token_account,
};
use spl_token::{instruction::initialize_mint, native_mint, state::Mint};

use crate::{
    config::{parse_keypair, parse_solana_configuration},
    schema::{CreateRewardCenterParams, PayoutOperation},
};

#[must_use]
pub fn generate_create_auction_house_ix(treasury_mint: Pubkey, payer: Pubkey) -> Instruction {
    let create_auction_house_accounts = CreateAuctionHouseAccounts {
        treasury_mint,
        payer,
        authority: payer,
        fee_withdrawal_destination: payer,
        treasury_withdrawal_destination: payer,
        treasury_withdrawal_destination_owner: payer,
    };

    let create_auction_house_data = CreateAuctionHouseData {
        seller_fee_basis_points: 100,
        requires_sign_off: false,
        can_change_sale_price: false,
    };

    create_auction_house(create_auction_house_accounts, create_auction_house_data)
}

/// # Errors
///
/// Will return `Err` if rent exemption fetch fails
pub fn generate_create_rewards_mint_ixs(
    client: &RpcClient,
    rewards_mint_authority: &Pubkey,
    rewards_mint: &Pubkey,
) -> AnyhowResult<Vec<Instruction>> {
    let token_program = spl_token::id();

    // Assign account and rent
    let mint_account_rent = client
        .get_minimum_balance_for_rent_exemption(Mint::LEN)
        .context("Unable to fetch rent exemption lamports")?;

    let allocate_rewards_mint_space_ix = create_account(
        rewards_mint_authority,
        rewards_mint,
        mint_account_rent,
        Mint::LEN as u64,
        &token_program,
    );

    // Initialize rewards mint
    let init_rewards_reward_mint_ix = initialize_mint(
        &token_program,
        rewards_mint,
        rewards_mint_authority,
        Some(rewards_mint_authority),
        9,
    )?;

    // Create token account for mint authority
    let mint_auth_rewards_mint_token_account =
        get_associated_token_address(rewards_mint_authority, rewards_mint);

    let create_associated_token_mint_auth_ix = create_associated_token_account(
        rewards_mint_authority,
        &mint_auth_rewards_mint_token_account,
        rewards_mint,
        &spl_token::ID,
    );

    Ok(vec![
        allocate_rewards_mint_space_ix,
        init_rewards_reward_mint_ix,
        create_associated_token_mint_auth_ix,
    ])
}

#[must_use]
pub fn generate_delegate_auctioneer_ix(
    auction_house: Pubkey,
    authority: Pubkey,
    reward_center: Pubkey,
) -> Instruction {
    let delegate_auctioneer_accounts = DelegateAuctioneerAccounts {
        auction_house,
        authority,
        auctioneer_authority: reward_center,
    };

    let delegate_auctioneer_data = DelegateAuctioneerData {
        scopes: vec![
            AuthorityScope::Buy,
            AuthorityScope::Cancel,
            AuthorityScope::Sell,
            AuthorityScope::Withdraw,
            AuthorityScope::PublicBuy,
            AuthorityScope::Deposit,
            AuthorityScope::ExecuteSale,
        ],
    };

    delegate_auctioneer(delegate_auctioneer_accounts, delegate_auctioneer_data)
}

#[must_use]
pub fn generate_rent_exempt_ixs(
    auction_house: Pubkey,
    authority: Pubkey,
    rent_amount: u64,
) -> (Instruction, Instruction) {
    let (auction_house_treasury, _auction_house_treasury_bump) =
        find_auction_house_treasury_address(&auction_house);

    let auction_house_treasury_rent_exempt_ix =
        transfer(&authority, &auction_house_treasury, rent_amount);

    let (auction_house_fee_account, _auction_house_fee_account_bump) =
        find_auction_house_fee_account_address(&auction_house);

    let auction_house_fee_account_rent_exempt_ix =
        transfer(&authority, &auction_house_fee_account, rent_amount);

    (
        auction_house_treasury_rent_exempt_ix,
        auction_house_fee_account_rent_exempt_ix,
    )
}

#[must_use]
pub fn generate_create_reward_center_ix(
    wallet: Pubkey,
    rewards_mint: Pubkey,
    auction_house: Pubkey,
    CreateRewardCenterParams {
        mathematical_operand,
        seller_reward_payout_basis_points,
        payout_numeral,
    }: CreateRewardCenterParams,
) -> Instruction {
    create_reward_center(
        CreateRewardCenterAccounts {
            wallet,
            mint: rewards_mint,
            auction_house,
            auction_house_treasury_mint: native_mint::id(),
        },
        hpl_reward_center::reward_centers::create::CreateRewardCenterParams {
            reward_rules: {
                hpl_reward_center::state::RewardRules {
                    seller_reward_payout_basis_points,
                    mathematical_operand: match mathematical_operand {
                        PayoutOperation::Divide => {
                            hpl_reward_center::state::PayoutOperation::Divide
                        },
                        PayoutOperation::Multiple => {
                            hpl_reward_center::state::PayoutOperation::Multiple
                        },
                    },
                    payout_numeral,
                }
            },
        },
    )
}

/// # Errors
///
/// Will return `Err` if the following happens
/// 1. Mint rewards/Auction House/Keypair Path fails to parse/open
/// 2. Transaction errors due to validation
/// 3. RPC Errors if timed out
pub fn process_create_reward_center(
    client: &RpcClient,
    keypair_path: &Option<PathBuf>,
    config_file: PathBuf,
    auction_house: &Option<String>,
    mint_rewards: &Option<String>,
) -> AnyhowResult<()> {
    let solana_options = parse_solana_configuration()?;

    let keypair = parse_keypair(keypair_path, &solana_options)?;

    let mut instructions: Vec<Instruction> = vec![];

    let auction_house_pubkey = match &auction_house {
        Some(cli_auction_house) => Pubkey::from_str(cli_auction_house)
            .context("Failed to parse Pubkey from mint rewards string")?,
        None => find_auction_house_address(&keypair.pubkey(), &native_mint::id()).0,
    };

    if auction_house.is_none() {
        info!(
            "Auction house account not passed. Creating a new auction house with default parameters of address {}", auction_house_pubkey.to_string()
        );

        let create_auction_house_ix =
            generate_create_auction_house_ix(native_mint::id(), keypair.pubkey());

        instructions.push(create_auction_house_ix);
    }

    let reward_mint_keypair = Keypair::new();
    let rewards_mint_pubkey = match &mint_rewards {
        Some(rewards_mint) => Pubkey::from_str(rewards_mint)
            .context("Failed to parse Pubkey from rewards mint string")?,
        None => reward_mint_keypair.pubkey(),
    };

    if mint_rewards.is_none() {
        info!("Rewards mint address not found. Creating a new mint.");
        let rewards_mint_authority_pubkey = keypair.pubkey();

        let mut create_rewards_mint_ixs = generate_create_rewards_mint_ixs(
            client,
            &rewards_mint_authority_pubkey,
            &rewards_mint_pubkey,
        )
        .context("Failed to generate reward mint instructions")?;

        instructions.append(&mut create_rewards_mint_ixs);
    }

    let create_reward_center_params = if Path::new(&config_file).exists() {
        let create_reward_center_config_file = File::open(config_file)?;
        serde_json::from_reader(create_reward_center_config_file)?
    } else {
        warn!("Create reward center config doesn't exist");
        CreateRewardCenterParams {
            mathematical_operand: PayoutOperation::Divide,
            payout_numeral: 5,
            seller_reward_payout_basis_points: 1000,
        }
    };

    let (reward_center_pubkey, _) = find_reward_center_address(&auction_house_pubkey);

    let create_reward_center_ix = generate_create_reward_center_ix(
        keypair.pubkey(),
        rewards_mint_pubkey,
        auction_house_pubkey,
        create_reward_center_params,
    );

    instructions.push(create_reward_center_ix);

    let delegate_auctioneer_ix = generate_delegate_auctioneer_ix(
        auction_house_pubkey,
        keypair.pubkey(),
        reward_center_pubkey,
    );

    instructions.push(delegate_auctioneer_ix);

    let rent_exempt = client.get_minimum_balance_for_rent_exemption(0)?;

    let (treasury_rent_exempt_ix, fee_account_rent_exempt_ix) =
        generate_rent_exempt_ixs(auction_house_pubkey, keypair.pubkey(), rent_exempt);

    instructions.push(treasury_rent_exempt_ix);
    instructions.push(fee_account_rent_exempt_ix);

    let latest_blockhash = client.get_latest_blockhash()?;

    let transaction = if mint_rewards.is_some() {
        Transaction::new_signed_with_payer(
            &instructions,
            Some(&keypair.pubkey()),
            &[&keypair],
            latest_blockhash,
        )
    } else {
        Transaction::new_signed_with_payer(
            &instructions,
            Some(&keypair.pubkey()),
            &[&keypair, &reward_mint_keypair],
            latest_blockhash,
        )
    };

    let tx_hash = client.send_and_confirm_transaction(&transaction);

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
        "Reward center address: {}\n",
        reward_center_pubkey.to_string()
    );

    if mint_rewards.is_none() {
        info!(
            "Rewards mint address: {}\n",
            rewards_mint_pubkey.to_string()
        );
    }

    Ok(())
}
