pub mod accounts;
pub mod args;

pub use accounts::*;
pub use args::*;

use anchor_client::solana_sdk::sysvar;
use anchor_client::solana_sdk::{instruction::Instruction, system_program};
use anchor_lang::{prelude::*, InstructionData};
use anchor_spl::{associated_token::AssociatedToken, token::spl_token};
use mpl_auction_house::pda::{
    find_auction_house_address, find_auction_house_fee_account_address,
    find_auction_house_treasury_address, find_auctioneer_pda,
};

pub fn create_auction_house(
    CreateAuctionHouseAccounts {
        treasury_mint,
        payer,
        authority,
        fee_withdrawal_destination,
        treasury_withdrawal_destination,
        treasury_withdrawal_destination_owner,
    }: CreateAuctionHouseAccounts,
    CreateAuctionHouseData {
        seller_fee_basis_points,
        requires_sign_off,
        can_change_sale_price,
    }: CreateAuctionHouseData,
) -> Instruction {
    let (auction_house, _bump) = find_auction_house_address(&authority, &treasury_mint);
    let (auction_house_fee_account, fee_payer_bump) =
        find_auction_house_fee_account_address(&auction_house);
    let (auction_house_treasury, treasury_bump) =
        find_auction_house_treasury_address(&auction_house);

    let accounts = mpl_auction_house::accounts::CreateAuctionHouse {
        treasury_mint,
        payer,
        authority,
        fee_withdrawal_destination,
        treasury_withdrawal_destination,
        treasury_withdrawal_destination_owner,
        auction_house,
        auction_house_fee_account,
        auction_house_treasury,
        token_program: spl_token::id(),
        system_program: system_program::id(),
        ata_program: AssociatedToken::id(),
        rent: sysvar::rent::id(),
    }
    .to_account_metas(None);

    let data = mpl_auction_house::instruction::CreateAuctionHouse {
        _bump,
        fee_payer_bump,
        treasury_bump,
        seller_fee_basis_points,
        requires_sign_off,
        can_change_sale_price,
    }
    .data();

    Instruction {
        program_id: mpl_auction_house::id(),
        data,
        accounts,
    }
}

pub fn delegate_auctioneer(
    DelegateAuctioneerAccounts {
        auction_house,
        authority,
        auctioneer_authority,
    }: DelegateAuctioneerAccounts,
    DelegateAuctioneerData { scopes }: DelegateAuctioneerData,
) -> Instruction {
    let (ah_auctioneer_pda, _) = find_auctioneer_pda(&auction_house, &auctioneer_authority);

    let accounts = mpl_auction_house::accounts::DelegateAuctioneer {
        auction_house,
        authority,
        auctioneer_authority,
        ah_auctioneer_pda,
        system_program: system_program::id(),
    }
    .to_account_metas(None);

    let data = mpl_auction_house::instruction::DelegateAuctioneer { scopes }.data();

    Instruction {
        program_id: mpl_auction_house::id(),
        accounts,
        data,
    }
}

pub fn withdraw_from_treasury(
    WithdrawFromTreasuryAccounts {
        auction_house,
        authority,
        treasury_mint,
        treasury_withdrawal_destination,
    }: WithdrawFromTreasuryAccounts,
    withdrawal_amount: u64,
) -> Instruction {
    let (auction_house_treasury, _) = find_auction_house_treasury_address(&auction_house);

    let accounts = mpl_auction_house::accounts::WithdrawFromTreasury {
        treasury_mint,
        treasury_withdrawal_destination,
        auction_house,
        auction_house_treasury,
        authority,
        system_program: system_program::id(),
        token_program: spl_token::id(),
    }
    .to_account_metas(None);

    let data = mpl_auction_house::instruction::WithdrawFromTreasury {
        amount: withdrawal_amount,
    }
    .data();

    Instruction {
        program_id: mpl_auction_house::id(),
        accounts,
        data,
    }
}
