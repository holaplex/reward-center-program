pub mod constants;
pub mod errors;
pub mod execute_sale;
pub mod listings;
pub mod metaplex_cpi;
pub mod offers;
pub mod pda;
pub mod reward_centers;
pub mod state;

use anchor_lang::prelude::*;
use mpl_auction_house::instruction::AuctioneerSell as AuctioneerSellParams;

use crate::{
    execute_sale::*,
<<<<<<< HEAD
    listings::{buy::*, close::*, create::*, update::*},
    offers::{close::*, create::*},
=======
    listings::{close::*, create::*, update::*},
    offers::{accept::*, close::*, create::*},
>>>>>>> 810f5d8 (fix: folder restructure)
    reward_centers::{create::*, edit::*},
};

declare_id!("RwDDvPp7ta9qqUwxbBfShsNreBaSsKvFcHzMxfBC3Ki");

#[program]
pub mod reward_center {
    use super::*;

    pub fn create_reward_center(
        ctx: Context<CreateRewardCenter>,
        create_reward_center_params: CreateRewardCenterParams,
    ) -> Result<()> {
        reward_centers::create::handler(ctx, create_reward_center_params)
    }

    pub fn edit_reward_center(
        ctx: Context<EditRewardCenter>,
        edit_reward_center_params: EditRewardCenterParams,
    ) -> Result<()> {
        reward_centers::edit::handler(ctx, edit_reward_center_params)
    }

    pub fn create_listing(
        ctx: Context<CreateListing>,
        create_listing_params: CreateListingParams,
    ) -> Result<()> {
        listings::create::handler(ctx, create_listing_params)
    }

    pub fn update_listing(
        ctx: Context<UpdateListing>,
        update_listing_params: UpdateListingParams,
    ) -> Result<()> {
        listings::update::handler(ctx, update_listing_params)
    }

    pub fn close_listing(ctx: Context<CloseListing>) -> Result<()> {
        listings::close::handler(ctx)
    }

    pub fn create_offer(
        ctx: Context<CreateOffer>,
        create_offer_params: CreateOfferParams,
    ) -> Result<()> {
        offers::create::handler(ctx, create_offer_params)
    }

    pub fn close_offer(
        ctx: Context<CloseOffer>,
        close_offer_params: CloseOfferParams,
    ) -> Result<()> {
        offers::close::handler(ctx, close_offer_params)
    }

    pub fn accept_offer(
        ctx: Context<AcceptOffer>,
        execute_sale_params: ExecuteSaleParams,
        auctioneer_sell_params: AuctioneerSellParams,
    ) -> Result<()> {
        offers::accept::handler(ctx, execute_sale_params, auctioneer_sell_params)
    }

    pub fn execute_sale(
        ctx: Context<ExecuteSale>,
        execute_sale_params: ExecuteSaleParams,
    ) -> Result<()> {
        execute_sale::handler(ctx, execute_sale_params)
    }

    pub fn buy_listing<'info>(
        ctx: Context<'_, '_, '_, 'info, BuyListing<'info>>,
        buy_listing_params: BuyListingParams,
    ) -> Result<()> {
        listings::buy::handler(ctx, buy_listing_params)
    }

    pub fn accept_offer(
        ctx: Context<AcceptOffer>,
        execute_sale_params: ExecuteSaleParams,
        auctioneer_sell_params: AuctioneerSellParams,
    ) -> Result<()> {
        accept_offer::handler(ctx, execute_sale_params, auctioneer_sell_params)
    }
}
