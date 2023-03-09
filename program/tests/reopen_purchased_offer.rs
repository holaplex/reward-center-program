#![cfg(feature = "test-bpf")]

pub mod reward_center_test;
use anchor_client::solana_sdk::{
    instruction::AccountMeta, pubkey::Pubkey, signature::Signer, transaction::Transaction,
};
use hpl_reward_center::{
    pda::{find_listing_address, find_reward_center_address},
    reward_centers,
    state::*,
};
use mpl_auction_house::{
    pda::{
        find_auction_house_address, find_auction_house_fee_account_address,
        find_auctioneer_trade_state_address, find_trade_state_address,
    },
    AuthorityScope,
};
use reward_center_test::fixtures::metadata;

use hpl_reward_center_sdk::{
    accounts::{AcceptOfferAccounts, *},
    args::{AcceptOfferData, *},
    *,
};

use mpl_testing_utils::solana::airdrop;
use solana_program_test::*;
use solana_sdk::{program_pack::Pack, signature::Keypair, system_instruction::create_account};
use std::str::FromStr;

use mpl_token_metadata::state::Collection;

use spl_associated_token_account::{create_associated_token_account, get_associated_token_address};
use spl_token::{
    instruction::{initialize_mint, mint_to_checked, transfer_checked as token_transfer},
    native_mint,
    state::Mint,
};

#[tokio::test]
async fn reopen_purchased_offer_success() {
    let program = reward_center_test::setup_program();
    let mut context = program.start_with_context().await;
    let rent = context.banks_client.get_rent().await.unwrap();
    let wallet = context.payer.pubkey();
    let mint = native_mint::id();
    let collection = Pubkey::from_str(reward_center_test::TEST_COLLECTION).unwrap();

    let metadata = metadata::create(
        &mut context,
        metadata::Params {
            name: "Test",
            symbol: "TST",
            uri: "https://nfts.exp.com/1.json",
            seller_fee_basis_points: 10,
            is_mutable: false,
            collection: Some(Collection {
                verified: false,
                key: collection,
            }),
            uses: None,
        },
        None,
    )
    .await;

    let metadata_owner = metadata.token;
    let metadata_address = metadata.pubkey;
    let metadata_owner_address = metadata_owner.pubkey();
    let metadata_mint_address = metadata.mint.pubkey();

    let (auction_house, _) = find_auction_house_address(&wallet, &mint);
    let (reward_center, _) = find_reward_center_address(&auction_house);
    let (listing, _) =
        find_listing_address(&metadata_owner_address, &metadata_address, &reward_center);

    // Creating Rewards mint and token account
    let token_program = &spl_token::id();
    let reward_mint_authority_keypair = Keypair::new();
    let reward_mint_keypair = Keypair::new();

    let reward_mint_authority_pubkey = reward_mint_authority_keypair.pubkey();
    let reward_mint_pubkey = reward_mint_keypair.pubkey();

    airdrop(
        &mut context,
        &reward_mint_authority_pubkey,
        reward_center_test::TEN_SOL,
    )
    .await
    .unwrap();

    // Assign account and rent
    let mint_account_rent = rent.minimum_balance(Mint::LEN);
    let allocate_reward_mint_space_ix = create_account(
        &reward_mint_authority_pubkey,
        &reward_mint_pubkey,
        mint_account_rent,
        Mint::LEN as u64,
        &token_program,
    );

    // Initialize rewards mint
    let init_rewards_reward_mint_ix = initialize_mint(
        &token_program,
        &reward_mint_pubkey,
        &reward_mint_authority_pubkey,
        Some(&reward_mint_authority_pubkey),
        9,
    )
    .unwrap();

    // Minting initial tokens to reward_center
    let reward_center_reward_token_account =
        get_associated_token_address(&reward_center, &reward_mint_pubkey);

    let mint_reward_tokens_ix = mint_to_checked(
        &token_program,
        &reward_mint_pubkey,
        &reward_center_reward_token_account,
        &reward_mint_authority_pubkey,
        &[],
        100_000_000_000,
        9,
    )
    .unwrap();

    let reward_center_params = reward_centers::create::CreateRewardCenterParams {
        reward_rules: RewardRules {
            mathematical_operand: PayoutOperation::Divide,
            seller_reward_payout_basis_points: 1000,
            payout_numeral: 5,
        },
    };

    let create_auction_house_accounts = mpl_auction_house_sdk::CreateAuctionHouseAccounts {
        treasury_mint: mint,
        payer: wallet,
        authority: wallet,
        fee_withdrawal_destination: wallet,
        treasury_withdrawal_destination: wallet,
        treasury_withdrawal_destination_owner: wallet,
    };
    let create_auction_house_data = mpl_auction_house_sdk::CreateAuctionHouseData {
        seller_fee_basis_points: 100,
        requires_sign_off: false,
        can_change_sale_price: false,
    };

    let create_auction_house_ix = mpl_auction_house_sdk::create_auction_house(
        create_auction_house_accounts,
        create_auction_house_data,
    );

    let create_reward_center_ix = hpl_reward_center_sdk::create_reward_center(
        hpl_reward_center_sdk::accounts::CreateRewardCenterAccounts {
            wallet,
            mint: reward_mint_keypair.pubkey(),
            auction_house_treasury_mint: mint,
            auction_house,
        },
        reward_center_params,
    );

    let delegate_auctioneer_accounts = mpl_auction_house_sdk::DelegateAuctioneerAccounts {
        auction_house,
        authority: wallet,
        auctioneer_authority: reward_center,
    };

    let delegate_auctioneer_data = mpl_auction_house_sdk::DelegateAuctioneerData {
        scopes: vec![
            AuthorityScope::Deposit,
            AuthorityScope::Buy,
            AuthorityScope::PublicBuy,
            AuthorityScope::ExecuteSale,
            AuthorityScope::Sell,
            AuthorityScope::Cancel,
            AuthorityScope::Withdraw,
        ],
    };

    let delegate_auctioneer_ix = mpl_auction_house_sdk::delegate_auctioneer(
        delegate_auctioneer_accounts,
        delegate_auctioneer_data,
    );

    let token_account =
        get_associated_token_address(&metadata_owner_address, &metadata_mint_address);

    let (seller_trade_state, trade_state_bump) = find_auctioneer_trade_state_address(
        &metadata_owner_address,
        &auction_house,
        &token_account,
        &mint,
        &metadata_mint_address,
        1,
    );

    let (free_seller_trade_state, free_trade_state_bump) = find_trade_state_address(
        &metadata_owner_address,
        &auction_house,
        &token_account,
        &mint,
        &metadata_mint_address,
        0,
        1,
    );

    let tx = Transaction::new_signed_with_payer(
        &[
            create_auction_house_ix,
            allocate_reward_mint_space_ix,
            init_rewards_reward_mint_ix,
            create_reward_center_ix,
            mint_reward_tokens_ix,
            delegate_auctioneer_ix,
        ],
        Some(&wallet),
        &[
            &context.payer,
            &reward_mint_authority_keypair,
            &reward_mint_keypair,
        ],
        context.last_blockhash,
    );

    let tx_response = context.banks_client.process_transaction(tx).await;

    assert!(tx_response.is_ok());

    // CREATE OFFER TEST

    let buyer = Keypair::new();
    let buyer_pubkey = &buyer.pubkey();
    airdrop(&mut context, buyer_pubkey, reward_center_test::TEN_SOL)
        .await
        .unwrap();

    let create_offer_accounts = CreateOfferAccounts {
        wallet: *buyer_pubkey,
        transfer_authority: *buyer_pubkey,
        payment_account: *buyer_pubkey,
        treasury_mint: mint,
        token_mint: metadata_mint_address,
        auction_house,
        reward_center,
        token_account,
        metadata: metadata_address,
        authority: wallet,
    };

    let create_offer_params = CreateOfferData {
        token_size: 1,
        buyer_price: reward_center_test::ONE_SOL,
    };

    let create_offer_ix = create_offer(create_offer_accounts, create_offer_params);

    let tx = Transaction::new_signed_with_payer(
        &[create_offer_ix],
        Some(buyer_pubkey),
        &[&buyer],
        context.last_blockhash,
    );

    let tx_response = context.banks_client.process_transaction(tx).await;

    assert!(tx_response.is_ok());

    // ACCEPT OFFER

    let auction_house_fee_account = &find_auction_house_fee_account_address(&auction_house).0;

    airdrop(
        &mut context,
        auction_house_fee_account,
        reward_center_test::ONE_SOL,
    )
    .await
    .unwrap();

    // Creating Associated Token accounts
    let create_buyer_reward_token_ix = create_associated_token_account(
        &metadata_owner_address,
        &buyer_pubkey,
        &reward_mint_pubkey,
    );

    let create_seller_reward_token_ix = create_associated_token_account(
        &metadata_owner_address,
        &metadata_owner_address,
        &reward_mint_pubkey,
    );

    let buyer_token_account = get_associated_token_address(&buyer.pubkey(), &metadata_mint_address);

    let accept_offer_accounts = AcceptOfferAccounts {
        auction_house,
        token_account,
        buyer: buyer.pubkey(),
        seller: metadata_owner.pubkey(),
        authority: wallet,
        token_mint: metadata_mint_address,
        treasury_mint: mint,
        buyer_receipt_token_account: buyer_token_account,
        seller_payment_receipt_account: metadata_owner.pubkey(),
        metadata: metadata_address,
    };

    let accept_offer_data = AcceptOfferData {
        price: reward_center_test::ONE_SOL,
        token_size: 1,
        reward_mint: reward_mint_pubkey,
    };

    let accept_offer = accept_offer(
        accept_offer_accounts,
        accept_offer_data,
        vec![AccountMeta::new(metadata_owner_address, false)],
    );

    let tx = Transaction::new_signed_with_payer(
        &[
            create_buyer_reward_token_ix,
            create_seller_reward_token_ix,
            accept_offer,
        ],
        Some(&metadata_owner.pubkey()),
        &[&metadata_owner],
        context.last_blockhash,
    );

    let tx_response = context.banks_client.process_transaction(tx).await;

    assert!(tx_response.is_ok());

    // Resending NFT to seller
    let resend_nft_to_seller_ix = token_transfer(
        token_program,
        &buyer_token_account,
        &metadata_mint_address,
        &token_account,
        buyer_pubkey,
        &[],
        1,
        0,
    )
    .unwrap();

    // REOPENING LISTING TEST
    let reopen_listing_accounts = CreateListingAccounts {
        wallet: metadata_owner.pubkey(),
        listing,
        reward_center,
        token_account,
        metadata: metadata.pubkey,
        authority: wallet,
        auction_house,
        seller_trade_state,
        free_seller_trade_state,
    };

    let reopen_listing_params = CreateListingData {
        price: reward_center_test::ONE_SOL,
        token_size: 1,
        trade_state_bump,
        free_trade_state_bump,
    };

    let reopen_listing_ix = create_listing(reopen_listing_accounts, reopen_listing_params);

    // REOPEN PURCHASED OFFER TEST

    let reopen_offer_accounts = CreateOfferAccounts {
        wallet: *buyer_pubkey,
        transfer_authority: *buyer_pubkey,
        payment_account: *buyer_pubkey,
        treasury_mint: mint,
        token_mint: metadata_mint_address,
        auction_house,
        reward_center,
        token_account,
        metadata: metadata_address,
        authority: wallet,
    };

    let reopen_offer_params = CreateOfferData {
        token_size: 1,
        buyer_price: reward_center_test::ONE_SOL,
    };

    let reopen_offer_ix = create_offer(reopen_offer_accounts, reopen_offer_params);

    let tx = Transaction::new_signed_with_payer(
        &[resend_nft_to_seller_ix, reopen_listing_ix, reopen_offer_ix],
        Some(buyer_pubkey),
        &[&buyer, &metadata_owner],
        context.last_blockhash,
    );

    let tx_response = context.banks_client.process_transaction(tx).await;

    assert!(tx_response.is_ok());

    ()
}
