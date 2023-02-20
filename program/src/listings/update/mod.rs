use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;
use mpl_auction_house::{
    constants::PREFIX, program::AuctionHouse as AuctionHouseProgram, utils::assert_metadata_valid,
    AuctionHouse,
};

use crate::{
    constants::{LISTING, REWARD_CENTER},
    errors::RewardCenterError,
    state::{Listing, RewardCenter},
};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct UpdateListingParams {
    pub new_price: u64,
}

#[derive(Accounts, Clone)]
#[instruction(update_listing_params: UpdateListingParams)]
pub struct UpdateListing<'info> {
    /// Seller wallet
    #[account(mut, address = listing.seller)]
    pub wallet: Signer<'info>,

    /// The Listing Config used for listing settings
    #[account(
        mut,
        has_one = metadata,
        has_one = reward_center,
        seeds = [
            LISTING.as_bytes(),
            wallet.key().as_ref(),
            metadata.key().as_ref(),
            reward_center.key().as_ref(),
        ],
        constraint = update_listing_params.new_price > 0 @ RewardCenterError::PriceInvalid,
        bump = listing.bump,
    )]
    pub listing: Account<'info, Listing>,

    /// The auctioneer authority PDA running this auction.
    #[account(
        has_one = auction_house,
        seeds = [
            REWARD_CENTER.as_bytes(),
            auction_house.key().as_ref()
        ],
        bump = reward_center.bump,
    )]
    pub reward_center: Box<Account<'info, RewardCenter>>,

    /// Auction House instance PDA account.
    #[account(
        seeds = [
            PREFIX.as_bytes(),
            auction_house.creator.as_ref(),
            auction_house.treasury_mint.as_ref()
        ],
        seeds::program = auction_house_program,
        bump = auction_house.bump,
    )]
    pub auction_house: Box<Account<'info, AuctionHouse>>,

    /// CHECK: assertion with mpl_auction_house assert_metadata_valid
    /// Metaplex metadata account decorating SPL mint account.
    pub metadata: UncheckedAccount<'info>,

    /// SPL token account containing token for sale.
    #[account(
        constraint = token_account.owner == wallet.key(),
        constraint = token_account.amount == 1
    )]
    pub token_account: Box<Account<'info, TokenAccount>>,

    /// Auction House Program used for CPI call
    pub auction_house_program: Program<'info, AuctionHouseProgram>,
}

pub fn handler(
    ctx: Context<UpdateListing>,
    UpdateListingParams { new_price }: UpdateListingParams,
) -> Result<()> {
    let listing = &mut ctx.accounts.listing;
    let metadata = &ctx.accounts.metadata;
    let token_account = &ctx.accounts.token_account;

    assert_metadata_valid(metadata, token_account)?;

    listing.price = new_price;

    Ok(())
}
