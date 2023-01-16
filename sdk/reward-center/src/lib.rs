pub mod accounts;
pub mod args;

pub use accounts::*;

use anchor_client::solana_sdk::{instruction::Instruction, pubkey::Pubkey, system_program, sysvar};
use anchor_lang::{prelude::*, solana_program::instruction::AccountMeta, InstructionData};
use args::*;
use hpl_reward_center::{
    accounts as rewards_accounts, id, instruction,
    listings::{buy::BuyListingParams, create::CreateListingParams, update::UpdateListingParams},
    offers::{accept::AcceptOfferParams, close::CloseOfferParams, create::CreateOfferParams},
    pda::{self, find_listing_address, find_offer_address, find_reward_center_address},
    reward_centers::{create::CreateRewardCenterParams, edit::EditRewardCenterParams},
    withdraw::reward_center::WithdrawRewardCenterFundsParams,
};
use mpl_auction_house::pda::{
    find_auction_house_treasury_address, find_auctioneer_trade_state_address,
    find_public_bid_trade_state_address, find_trade_state_address,
};
use spl_associated_token_account::get_associated_token_address;

pub fn create_reward_center(
    CreateRewardCenterAccounts {
        wallet,
        mint,
        auction_house,
        auction_house_treasury_mint,
    }: CreateRewardCenterAccounts,
    create_reward_center_params: CreateRewardCenterParams,
) -> Instruction {
    let (reward_center, _) = pda::find_reward_center_address(&auction_house);
    let associated_token_account = get_associated_token_address(&reward_center, &mint);

    let accounts = rewards_accounts::CreateRewardCenter {
        wallet,
        mint,
        auction_house,
        reward_center,
        associated_token_account,
        auction_house_treasury_mint,
        token_program: spl_token::id(),
        associated_token_program: spl_associated_token_account::id(),
        rent: sysvar::rent::id(),
        system_program: system_program::id(),
    }
    .to_account_metas(None);

    let data = instruction::CreateRewardCenter {
        create_reward_center_params,
    }
    .data();

    Instruction {
        program_id: id(),
        accounts,
        data,
    }
}

pub fn edit_reward_center(
    wallet: Pubkey,
    auction_house: Pubkey,
    edit_reward_center_params: EditRewardCenterParams,
) -> Instruction {
    let (reward_center, _) = pda::find_reward_center_address(&auction_house);

    let accounts = rewards_accounts::EditRewardCenter {
        wallet,
        auction_house,
        reward_center,
    }
    .to_account_metas(None);

    let data = instruction::EditRewardCenter {
        edit_reward_center_params,
    }
    .data();

    Instruction {
        program_id: id(),
        accounts,
        data,
    }
}

pub fn withdraw_reward_center_funds(
    WithdrawRewardCenterFundsAccounts {
        wallet,
        auction_house,
        rewards_mint,
    }: WithdrawRewardCenterFundsAccounts,
    withdrawal_amount: u64,
) -> Instruction {
    let (reward_center, _) = pda::find_reward_center_address(&auction_house);

    let reward_center_reward_token_account =
        get_associated_token_address(&reward_center, &rewards_mint);

    let destination_reward_token_account = get_associated_token_address(&wallet, &rewards_mint);

    let accounts = rewards_accounts::WithdrawRewardCenterFunds {
        wallet,
        auction_house,
        reward_center,
        reward_center_reward_token_account,
        destination_reward_token_account,
        token_program: spl_token::id(),
    }
    .to_account_metas(None);

    let data = instruction::WithdrawRewardCenterFunds {
        withdraw_reward_center_funds_params: WithdrawRewardCenterFundsParams { withdrawal_amount },
    }
    .data();

    Instruction {
        program_id: id(),
        accounts,
        data,
    }
}

pub fn create_listing(
    CreateListingAccounts {
        wallet,
        listing,
        reward_center,
        token_account,
        metadata,
        authority,
        auction_house,
        seller_trade_state,
        free_seller_trade_state,
    }: CreateListingAccounts,
    CreateListingData {
        price,
        token_size,
        trade_state_bump,
        free_trade_state_bump,
    }: CreateListingData,
) -> Instruction {
    let (auction_house_fee_account, _) =
        mpl_auction_house::pda::find_auction_house_fee_account_address(&auction_house);
    let (ah_auctioneer_pda, _) =
        mpl_auction_house::pda::find_auctioneer_pda(&auction_house, &reward_center);
    let (program_as_signer, program_as_signer_bump) =
        mpl_auction_house::pda::find_program_as_signer_address();

    let accounts = rewards_accounts::CreateListing {
        auction_house_program: mpl_auction_house::id(),
        listing,
        reward_center,
        wallet,
        token_account,
        metadata,
        authority,
        auction_house,
        auction_house_fee_account,
        seller_trade_state,
        free_seller_trade_state,
        ah_auctioneer_pda,
        program_as_signer,
        token_program: spl_token::id(),
        system_program: system_program::id(),
        rent: sysvar::rent::id(),
    }
    .to_account_metas(None);

    let data = instruction::CreateListing {
        create_listing_params: CreateListingParams {
            price,
            token_size,
            trade_state_bump,
            free_trade_state_bump,
            program_as_signer_bump,
        },
    }
    .data();

    Instruction {
        program_id: id(),
        accounts,
        data,
    }
}

pub fn close_listing(
    CloseListingAccounts {
        auction_house,
        listing,
        reward_center,
        authority,
        metadata,
        token_account,
        token_mint,
        treasury_mint,
        wallet,
    }: CloseListingAccounts,
    CloseListingData { token_size }: CloseListingData,
) -> Instruction {
    let (auction_house_fee_account, _) =
        mpl_auction_house::pda::find_auction_house_fee_account_address(&auction_house);
    let (ah_auctioneer_pda, _) =
        mpl_auction_house::pda::find_auctioneer_pda(&auction_house, &reward_center);

    let (seller_trade_state, _) = find_auctioneer_trade_state_address(
        &wallet,
        &auction_house,
        &token_account,
        &treasury_mint,
        &token_mint,
        token_size,
    );

    let accounts = rewards_accounts::CloseListing {
        ah_auctioneer_pda,
        auction_house,
        auction_house_fee_account,
        authority,
        listing,
        metadata,
        reward_center,
        token_account,
        token_mint,
        trade_state: seller_trade_state,
        wallet,
        auction_house_program: mpl_auction_house::id(),
        token_program: spl_token::id(),
    }
    .to_account_metas(None);

    let data = instruction::CloseListing {}.data();

    Instruction {
        program_id: id(),
        accounts,
        data,
    }
}

pub fn update_listing(
    UpdateListingAccounts {
        auction_house,
        metadata,
        token_account,
        wallet,
    }: UpdateListingAccounts,
    UpdateListingData { new_price }: UpdateListingData,
) -> Instruction {
    let (reward_center, _) = find_reward_center_address(&auction_house);
    let (listing, _) = find_listing_address(&wallet, &metadata, &reward_center);

    let accounts = rewards_accounts::UpdateListing {
        auction_house,
        listing,
        metadata,
        reward_center,
        wallet,
        token_account,
        auction_house_program: mpl_auction_house::id(),
    }
    .to_account_metas(None);

    let data = instruction::UpdateListing {
        update_listing_params: UpdateListingParams { new_price },
    }
    .data();

    Instruction {
        program_id: id(),
        accounts,
        data,
    }
}

pub fn create_offer(
    CreateOfferAccounts {
        auction_house,
        authority,
        metadata,
        payment_account,
        reward_center,
        token_account,
        transfer_authority,
        treasury_mint,
        token_mint,
        wallet,
    }: CreateOfferAccounts,
    CreateOfferData {
        buyer_price,
        token_size,
    }: CreateOfferData,
) -> Instruction {
    let (auction_house_fee_account, _) =
        mpl_auction_house::pda::find_auction_house_fee_account_address(&auction_house);
    let (ah_auctioneer_pda, _) =
        mpl_auction_house::pda::find_auctioneer_pda(&auction_house, &reward_center);

    let (escrow_payment_account, escrow_payment_bump) =
        mpl_auction_house::pda::find_escrow_payment_address(&auction_house, &wallet);

    let (buyer_trade_state, trade_state_bump) = find_public_bid_trade_state_address(
        &wallet,
        &auction_house,
        &treasury_mint,
        &token_mint,
        buyer_price,
        token_size,
    );

    let (offer, _) = pda::find_offer_address(&wallet, &metadata, &reward_center);

    let accounts = rewards_accounts::CreateOffer {
        ah_auctioneer_pda,
        auction_house,
        auction_house_fee_account,
        authority,
        buyer_trade_state,
        metadata,
        payment_account,
        reward_center,
        token_account,
        transfer_authority,
        treasury_mint,
        escrow_payment_account,
        wallet,
        offer,
        auction_house_program: mpl_auction_house::id(),
        token_program: spl_token::id(),
        system_program: system_program::id(),
        rent: sysvar::rent::id(),
    }
    .to_account_metas(None);

    let data = instruction::CreateOffer {
        create_offer_params: CreateOfferParams {
            buyer_price,
            escrow_payment_bump,
            token_size,
            trade_state_bump,
        },
    }
    .data();

    Instruction {
        program_id: id(),
        accounts,
        data,
    }
}

pub fn close_offer(
    CloseOfferAccounts {
        auction_house,
        authority,
        metadata,
        receipt_account,
        reward_center,
        token_account,
        token_mint,
        treasury_mint,
        wallet,
    }: CloseOfferAccounts,
    CloseOfferData {
        buyer_price,
        token_size,
    }: CloseOfferData,
) -> Instruction {
    let (auction_house_fee_account, _) =
        mpl_auction_house::pda::find_auction_house_fee_account_address(&auction_house);
    let (ah_auctioneer_pda, _) =
        mpl_auction_house::pda::find_auctioneer_pda(&auction_house, &reward_center);
    let (escrow_payment_account, escrow_payment_bump) =
        mpl_auction_house::pda::find_escrow_payment_address(&auction_house, &wallet);

    let (buyer_trade_state, _trade_state_bump) = find_public_bid_trade_state_address(
        &wallet,
        &auction_house,
        &treasury_mint,
        &token_mint,
        buyer_price,
        token_size,
    );

    let (offer, _) = pda::find_offer_address(&wallet, &metadata, &reward_center);

    let accounts = rewards_accounts::CloseOffer {
        wallet,
        ah_auctioneer_pda,
        auction_house,
        auction_house_fee_account,
        authority,
        escrow_payment_account,
        metadata,
        offer,
        receipt_account,
        reward_center,
        token_account,
        token_mint,
        trade_state: buyer_trade_state,
        treasury_mint,
        auction_house_program: mpl_auction_house::id(),
        ata_program: spl_associated_token_account::id(),
        token_program: spl_token::id(),
        system_program: system_program::id(),
        rent: sysvar::rent::id(),
    }
    .to_account_metas(None);

    let data = instruction::CloseOffer {
        close_offer_params: CloseOfferParams {
            escrow_payment_bump,
        },
    }
    .data();

    Instruction {
        program_id: id(),
        accounts,
        data,
    }
}

pub fn buy_listing(
    BuyListingAccounts {
        transfer_authority,
        payment_account,
        auction_house,
        seller,
        buyer,
        authority,
        treasury_mint,
        token_mint,
        token_account,
        metadata,
        seller_payment_receipt_account,
        buyer_receipt_token_account,
    }: BuyListingAccounts,
    BuyListingData {
        token_size,
        price,
        reward_mint,
    }: BuyListingData,
    creators: Vec<AccountMeta>,
) -> Instruction {
    let (reward_center, _) = find_reward_center_address(&auction_house);
    let (listing, _) = find_listing_address(&seller, &metadata, &reward_center);

    let (auction_house_fee_account, _) =
        mpl_auction_house::pda::find_auction_house_fee_account_address(&auction_house);
    let (auction_house_treasury, _) = find_auction_house_treasury_address(&auction_house);
    let (ah_auctioneer_pda, _) =
        mpl_auction_house::pda::find_auctioneer_pda(&auction_house, &reward_center);
    let (escrow_payment_account, escrow_payment_bump) =
        mpl_auction_house::pda::find_escrow_payment_address(&auction_house, &buyer);

    let reward_center_reward_token_account =
        get_associated_token_address(&reward_center, &reward_mint);
    let buyer_reward_token_account = get_associated_token_address(&buyer, &reward_mint);
    let seller_reward_token_account = get_associated_token_address(&seller, &reward_mint);

    let (buyer_trade_state, buyer_trade_state_bump) = find_public_bid_trade_state_address(
        &buyer,
        &auction_house,
        &treasury_mint,
        &token_mint,
        price,
        token_size,
    );

    let (free_seller_trade_state, free_trade_state_bump) = find_trade_state_address(
        &seller,
        &auction_house,
        &token_account,
        &treasury_mint,
        &token_mint,
        0,
        token_size,
    );

    let (seller_trade_state, seller_trade_state_bump) = find_auctioneer_trade_state_address(
        &seller,
        &auction_house,
        &token_account,
        &treasury_mint,
        &token_mint,
        token_size,
    );

    let (program_as_signer, program_as_signer_bump) =
        mpl_auction_house::pda::find_program_as_signer_address();

    let accounts = rewards_accounts::BuyListing {
        buyer,
        payment_account,
        transfer_authority,
        buyer_reward_token_account,
        seller,
        seller_reward_token_account,
        listing,
        authority,
        treasury_mint,
        token_mint,
        token_account,
        metadata,
        buyer_receipt_token_account,
        seller_payment_receipt_account,
        auction_house_fee_account,
        ah_auctioneer_pda,
        escrow_payment_account,
        reward_center,
        reward_center_reward_token_account,
        auction_house,
        auction_house_treasury,
        buyer_trade_state,
        free_seller_trade_state,
        seller_trade_state,
        program_as_signer,
        auction_house_program: mpl_auction_house::id(),
        ata_program: spl_associated_token_account::id(),
        token_program: spl_token::id(),
        system_program: system_program::id(),
        rent: sysvar::rent::id(),
    }
    .to_account_metas(None);

    let data = instruction::BuyListing {
        buy_listing_params: BuyListingParams {
            escrow_payment_bump,
            free_trade_state_bump,
            program_as_signer_bump,
            seller_trade_state_bump,
            buyer_trade_state_bump,
        },
    }
    .data();

    Instruction {
        program_id: id(),
        accounts: accounts.into_iter().chain(creators).collect(),
        data,
    }
}

pub fn accept_offer(
    AcceptOfferAccounts {
        auction_house,
        seller,
        buyer,
        authority,
        treasury_mint,
        token_mint,
        token_account,
        metadata,
        seller_payment_receipt_account,
        buyer_receipt_token_account,
    }: AcceptOfferAccounts,
    AcceptOfferData {
        token_size,
        price,
        reward_mint,
    }: AcceptOfferData,
    creators: Vec<AccountMeta>,
) -> Instruction {
    let (reward_center, _) = find_reward_center_address(&auction_house);
    let (offer, _) = find_offer_address(&buyer, &metadata, &reward_center);

    let (auction_house_fee_account, _) =
        mpl_auction_house::pda::find_auction_house_fee_account_address(&auction_house);
    let (auction_house_treasury, _) = find_auction_house_treasury_address(&auction_house);
    let (ah_auctioneer_pda, _) =
        mpl_auction_house::pda::find_auctioneer_pda(&auction_house, &reward_center);
    let (escrow_payment_account, escrow_payment_bump) =
        mpl_auction_house::pda::find_escrow_payment_address(&auction_house, &buyer);

    let reward_center_reward_token_account =
        get_associated_token_address(&reward_center, &reward_mint);
    let buyer_reward_token_account = get_associated_token_address(&buyer, &reward_mint);
    let seller_reward_token_account = get_associated_token_address(&seller, &reward_mint);

    let (buyer_trade_state, buyer_trade_state_bump) = find_public_bid_trade_state_address(
        &buyer,
        &auction_house,
        &treasury_mint,
        &token_mint,
        price,
        token_size,
    );

    let (free_seller_trade_state, free_trade_state_bump) = find_trade_state_address(
        &seller,
        &auction_house,
        &token_account,
        &treasury_mint,
        &token_mint,
        0,
        token_size,
    );

    let (seller_trade_state, seller_trade_state_bump) = find_auctioneer_trade_state_address(
        &seller,
        &auction_house,
        &token_account,
        &treasury_mint,
        &token_mint,
        token_size,
    );

    let (program_as_signer, program_as_signer_bump) =
        mpl_auction_house::pda::find_program_as_signer_address();

    let accounts = rewards_accounts::AcceptOffer {
        buyer,
        buyer_reward_token_account,
        seller,
        seller_reward_token_account,
        offer,
        authority,
        treasury_mint,
        token_mint,
        token_account,
        metadata,
        buyer_receipt_token_account,
        seller_payment_receipt_account,
        auction_house_fee_account,
        ah_auctioneer_pda,
        escrow_payment_account,
        reward_center,
        reward_center_reward_token_account,
        auction_house,
        auction_house_treasury,
        buyer_trade_state,
        free_seller_trade_state,
        seller_trade_state,
        program_as_signer,
        auction_house_program: mpl_auction_house::id(),
        ata_program: spl_associated_token_account::id(),
        token_program: spl_token::id(),
        system_program: system_program::id(),
        rent: sysvar::rent::id(),
    }
    .to_account_metas(None);

    let data = instruction::AcceptOffer {
        accept_offer_params: AcceptOfferParams {
            escrow_payment_bump,
            free_trade_state_bump,
            program_as_signer_bump,
            seller_trade_state_bump,
            buyer_trade_state_bump,
        },
    }
    .data();

    Instruction {
        program_id: id(),
        accounts: accounts.into_iter().chain(creators).collect(),
        data,
    }
}
