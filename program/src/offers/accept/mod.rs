use crate::constants::{LISTING, OFFER, REWARD_CENTER};
use crate::errors::RewardCenterError;
use crate::metaplex_cpi::auction_house::{make_auctioneer_instruction, AuctioneerInstructionArgs};
use crate::state::{Listing, Offer, RewardCenter};
use crate::{instruction::ExecuteSale, ExecuteSaleParams};
use anchor_lang::{
    prelude::*,
    solana_program::{instruction::Instruction, program::invoke},
    InstructionData,
};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};
use mpl_auction_house::constants::TREASURY;
use mpl_auction_house::{
    constants::{AUCTIONEER, FEE_PAYER, PREFIX, SIGNER},
    cpi::accounts::AuctioneerSell,
    instruction::AuctioneerSell as AuctioneerSellParams,
    program::AuctionHouse as AuctionHouseProgram,
    AuctionHouse, Auctioneer,
};
use solana_program::program::invoke_signed;

#[derive(Accounts, Clone)]
#[instruction(execute_sale_params: ExecuteSaleParams)]
pub struct AcceptOffer<'info> {
    // Accounts passed into Auction House CPI call
    /// CHECK: Verified through CPI
    /// Buyer user wallet account.
    #[account(mut)]
    pub buyer: UncheckedAccount<'info>,

    /// The token account to receive the buyer rewards.
    #[account(
        mut,
        constraint = reward_center.token_mint == buyer_reward_token_account.mint @ RewardCenterError::MintMismatch,
        constraint = buyer_reward_token_account.owner == buyer.key() @ RewardCenterError::BuyerTokenAccountMismatch,
    )]
    pub buyer_reward_token_account: Box<Account<'info, TokenAccount>>,

    /// CHECK: Verified through CPI
    /// Seller user wallet account.
    #[account(mut)]
    pub seller: UncheckedAccount<'info>,

    /// The token account to receive the seller rewards.
    #[account(
        mut,
        constraint = reward_center.token_mint == buyer_reward_token_account.mint @ RewardCenterError::MintMismatch,
        constraint = seller_reward_token_account.owner == seller.key() @ RewardCenterError::SellerTokenAccountMismatch,
    )]
    pub seller_reward_token_account: Box<Account<'info, TokenAccount>>,

    // Accounts used for Auctioneer
    /// The Listing Config used for listing settings
    #[account(
        mut,
        seeds = [
            LISTING.as_bytes(),
            seller.key().as_ref(),
            metadata.key().as_ref(),
            reward_center.key().as_ref(),
        ],
        bump = listing.bump,
        constraint = listing.price == offer.price @ RewardCenterError::PriceMismatch,
        close = seller,
    )]
    pub listing: Box<Account<'info, Listing>>,

    /// The offer config account used for bids
    #[account(
        mut,
        seeds = [
            OFFER.as_bytes(),
            buyer.key().as_ref(),
            metadata.key().as_ref(),
            reward_center.key().as_ref()
        ],
        bump = offer.bump,
        close = buyer,
    )]
    pub offer: Box<Account<'info, Offer>>,

    /// Payer account for initializing purchase_receipt_account
    #[account(mut)]
    pub payer: Signer<'info>,

    ///Token account where the SPL token is stored.
    #[account(
        mut,
        constraint = token_account.owner == seller.key(),
    )]
    pub token_account: Box<Account<'info, TokenAccount>>,

    /// Token mint account for the SPL token.
    pub token_mint: Box<Account<'info, Mint>>,

    /// CHECK: assertion with mpl_auction_house assert_metadata_valid
    /// Metaplex metadata account decorating SPL mint account.
    pub metadata: UncheckedAccount<'info>,

    /// Auction House treasury mint account.
    pub treasury_mint: Box<Account<'info, Mint>>,

    /// CHECK: Verified through CPI
    /// Seller SOL or SPL account to receive payment at.
    #[account(mut)]
    pub seller_payment_receipt_account: UncheckedAccount<'info>,

    /// CHECK: Verified through CPI
    /// Buyer SPL token account to receive purchased item at.
    #[account(mut)]
    pub buyer_receipt_token_account: UncheckedAccount<'info>,

    /// CHECK: Verified through CPI
    /// Auction House instance authority.
    pub authority: UncheckedAccount<'info>,

    /// CHECK: Not dangerous. Account seeds checked in constraint.
    /// Buyer escrow payment account.
    #[account(
        mut,
        seeds = [
            PREFIX.as_bytes(),
            auction_house.key().as_ref(),
            buyer.key().as_ref()
        ],
        seeds::program = auction_house_program,
        bump = execute_sale_params.escrow_payment_bump
    )]
    pub escrow_payment_account: UncheckedAccount<'info>,

    /// Auction House instance PDA account.
    #[account(
        seeds = [
            PREFIX.as_bytes(),
            auction_house.creator.as_ref(),
            auction_house.treasury_mint.as_ref()
        ],
        seeds::program = auction_house_program,
        bump = auction_house.bump,
        has_one = treasury_mint,
        has_one = auction_house_treasury,
        has_one = auction_house_fee_account
    )]
    pub auction_house: Box<Account<'info, AuctionHouse>>,

    /// CHECK: Not dangerous. Account seeds checked in constraint.
    /// Auction House instance fee account.
    #[account(
        mut,
        seeds = [
            PREFIX.as_bytes(),
            auction_house.key().as_ref(),
            FEE_PAYER.as_bytes()
        ],
        seeds::program = auction_house_program,
        bump = auction_house.fee_payer_bump
    )]
    pub auction_house_fee_account: UncheckedAccount<'info>,

    /// CHECK: Not dangerous. Account seeds checked in constraint.
    /// Auction House instance treasury account.
    #[account(
        mut,
        seeds = [
            PREFIX.as_bytes(),
            auction_house.key().as_ref(),
            TREASURY.as_bytes()
        ],
        seeds::program = auction_house_program,
        bump = auction_house.treasury_bump
    )]
    pub auction_house_treasury: UncheckedAccount<'info>,

    /// CHECK: Verified through CPI
    /// Buyer trade state PDA account encoding the buy order.
    #[account(mut)]
    pub buyer_trade_state: UncheckedAccount<'info>,

    /// CHECK: Not dangerous. Account seeds checked in constraint.
    /// Seller trade state PDA account encoding the sell order.
    #[account(
        mut,
        seeds = [
            PREFIX.as_bytes(),
            seller.key().as_ref(),
            auction_house.key().as_ref(),
            token_account.key().as_ref(),
            auction_house.treasury_mint.as_ref(),
            token_mint.key().as_ref(),
            &u64::MAX.to_le_bytes(),
            &listing.token_size.to_le_bytes()
        ],
        seeds::program = auction_house_program,
        bump = execute_sale_params.seller_trade_state_bump,
    )]
    pub seller_trade_state: UncheckedAccount<'info>,

    /// CHECK: Not dangerous. Account seeds checked in constraint.
    /// Free seller trade state PDA account encoding a free sell order.
    #[account(
        mut,
        seeds = [
            PREFIX.as_bytes(),
            seller.key().as_ref(),
            auction_house.key().as_ref(),
            token_account.key().as_ref(),
            auction_house.treasury_mint.as_ref(),
            token_account.mint.as_ref(),
            &0u64.to_le_bytes(),
            &listing.token_size.to_le_bytes()
        ],
        seeds::program = auction_house_program,
        bump = execute_sale_params.free_trade_state_bump
    )]
    pub free_seller_trade_state: UncheckedAccount<'info>,

    /// CHECK: Verified through CPI
    /// The auctioneer authority PDA running this auction.
    #[account(
        has_one = auction_house,
        seeds = [
            REWARD_CENTER.as_bytes(),
            auction_house.key().as_ref()
        ],
        bump = reward_center.bump
    )]
    pub reward_center: Box<Account<'info, RewardCenter>>,

    #[
        account(
            mut,
            constraint = reward_center.token_mint == reward_center_reward_token_account.mint @ RewardCenterError::MintMismatch
        )
    ]
    /// The token account holding the reward token for the reward center.
    pub reward_center_reward_token_account: Box<Account<'info, TokenAccount>>,

    /// CHECK: Not dangerous. Account seeds checked in constraint.
    /// The auctioneer PDA owned by Auction House storing scopes.
    #[account(
        seeds = [
            AUCTIONEER.as_bytes(),
            auction_house.key().as_ref(),
            reward_center.key().as_ref()
        ],
        seeds::program = auction_house_program,
        bump = ah_auctioneer_pda.bump
    )]
    pub ah_auctioneer_pda: Box<Account<'info, Auctioneer>>,

    /// CHECK: Not dangerous. Account seeds checked in constraint.
    #[account(
        seeds = [
            PREFIX.as_bytes(),
            SIGNER.as_bytes()
        ],
        seeds::program = auction_house_program,
        bump = execute_sale_params.program_as_signer_bump
    )]
    pub program_as_signer: UncheckedAccount<'info>,

    /// CHECK: Program for self-cpi
    #[account(
        constraint = program_itself.key() == crate::ID,
    )]
    pub program_itself: UncheckedAccount<'info>,
    /// Auction House Program
    pub auction_house_program: Program<'info, AuctionHouseProgram>,
    /// Token Program
    pub token_program: Program<'info, Token>,
    /// System Program
    pub system_program: Program<'info, System>,
    /// Associated Token Program
    pub ata_program: Program<'info, AssociatedToken>,
    /// Rent
    pub rent: Sysvar<'info, Rent>,
}

// V5: [227, 82, 234, 131, 1, 18, 48, 2]
// V6: [188, 168, 30, 200, 244, 230, 29, 230]
const EXECUTE_SALE_SIGHASH: [u8; 8] = [227, 82, 234, 131, 1, 18, 48, 2];

fn get_execute_sale_account_metas(ctx: &Context<AcceptOffer>) -> Vec<AccountMeta> {
    vec![
        ctx.accounts.buyer.to_account_metas(None),
        ctx.accounts
            .buyer_reward_token_account
            .to_account_metas(None),
        ctx.accounts.seller.to_account_metas(None),
        ctx.accounts
            .seller_reward_token_account
            .to_account_metas(None),
        ctx.accounts.listing.to_account_metas(None),
        ctx.accounts.offer.to_account_metas(None),
        ctx.accounts.payer.to_account_metas(None),
        ctx.accounts.token_account.to_account_metas(None),
        ctx.accounts.token_mint.to_account_metas(None),
        ctx.accounts.metadata.to_account_metas(None),
        ctx.accounts.treasury_mint.to_account_metas(None),
        ctx.accounts
            .seller_payment_receipt_account
            .to_account_metas(None),
        ctx.accounts
            .buyer_receipt_token_account
            .to_account_metas(None),
        ctx.accounts.authority.to_account_metas(None),
        ctx.accounts.escrow_payment_account.to_account_metas(None),
        ctx.accounts.auction_house.to_account_metas(None),
        ctx.accounts
            .auction_house_fee_account
            .to_account_metas(None),
        ctx.accounts.auction_house_treasury.to_account_metas(None),
        ctx.accounts.buyer_trade_state.to_account_metas(None),
        ctx.accounts.seller_trade_state.to_account_metas(None),
        ctx.accounts.free_seller_trade_state.to_account_metas(None),
        ctx.accounts.reward_center.to_account_metas(None),
        ctx.accounts
            .reward_center_reward_token_account
            .to_account_metas(None),
        ctx.accounts.ah_auctioneer_pda.to_account_metas(None),
    ]
    .iter()
    .flat_map(|v| v.iter().cloned())
    .collect::<Vec<AccountMeta>>()
}

pub fn handler(
    ctx: Context<AcceptOffer>,
    execute_sale_params: ExecuteSaleParams,
    auctioneer_sell_params: AuctioneerSellParams,
) -> Result<()> {
    let auction_house_key = ctx.accounts.auction_house.key();
    let reward_center_bump = ctx.accounts.reward_center.bump;
    let auctioneer_sell_ctx_accounts = AuctioneerSell {
        metadata: ctx.accounts.metadata.to_account_info(),
        wallet: ctx.accounts.seller.to_account_info(),
        token_account: ctx.accounts.token_account.to_account_info(),
        auction_house: ctx.accounts.auction_house.to_account_info(),
        auction_house_fee_account: ctx.accounts.auction_house_fee_account.to_account_info(),
        seller_trade_state: ctx.accounts.seller_trade_state.to_account_info(),
        free_seller_trade_state: ctx.accounts.free_seller_trade_state.to_account_info(),
        authority: ctx.accounts.authority.to_account_info(),
        auctioneer_authority: ctx.accounts.reward_center.to_account_info(),
        ah_auctioneer_pda: ctx.accounts.ah_auctioneer_pda.to_account_info(),
        token_program: ctx.accounts.token_program.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
        program_as_signer: ctx.accounts.program_as_signer.to_account_info(),
        rent: ctx.accounts.rent.to_account_info(),
    };

    let auctioneer_sell_signer_seeds: &[&[&[u8]]] = &[&[
        REWARD_CENTER.as_bytes(),
        auction_house_key.as_ref(),
        &[reward_center_bump],
    ]];

    let (sell_ix, sell_account_infos) = make_auctioneer_instruction(AuctioneerInstructionArgs {
        accounts: auctioneer_sell_ctx_accounts,
        instruction_data: auctioneer_sell_params.data(),
        auctioneer_authority: ctx.accounts.reward_center.key(),
    });

    invoke_signed(&sell_ix, &sell_account_infos, auctioneer_sell_signer_seeds)?;
    // Perform execute_sale_cpi
    let mut execute_sale_args = ExecuteSale {
        execute_sale_params,
    }
    .try_to_vec()?;
    let mut execute_sale_bytes: Vec<u8> = EXECUTE_SALE_SIGHASH.try_to_vec()?;
    execute_sale_bytes.append(&mut execute_sale_args);
    let execute_sale_ix = Instruction::new_with_bytes(
        crate::ID,
        &execute_sale_bytes,
        get_execute_sale_account_metas(&ctx),
    );
    invoke(
        &execute_sale_ix,
        &[
            ctx.accounts.buyer.to_account_info(),
            ctx.accounts.buyer_reward_token_account.to_account_info(),
            ctx.accounts.seller.to_account_info(),
            ctx.accounts.seller_reward_token_account.to_account_info(),
            ctx.accounts.listing.to_account_info(),
            ctx.accounts.offer.to_account_info(),
            ctx.accounts.payer.to_account_info(),
            ctx.accounts.token_account.to_account_info(),
            ctx.accounts.token_mint.to_account_info(),
            ctx.accounts.metadata.to_account_info(),
            ctx.accounts.treasury_mint.to_account_info(),
            ctx.accounts
                .seller_payment_receipt_account
                .to_account_info(),
            ctx.accounts.buyer_receipt_token_account.to_account_info(),
            ctx.accounts.authority.to_account_info(),
            ctx.accounts.escrow_payment_account.to_account_info(),
            ctx.accounts.auction_house.to_account_info(),
            ctx.accounts.auction_house_fee_account.to_account_info(),
            ctx.accounts.auction_house_treasury.to_account_info(),
            ctx.accounts.buyer_trade_state.to_account_info(),
            ctx.accounts.seller_trade_state.to_account_info(),
            ctx.accounts.free_seller_trade_state.to_account_info(),
            ctx.accounts.reward_center.to_account_info(),
            ctx.accounts
                .reward_center_reward_token_account
                .to_account_info(),
            ctx.accounts.ah_auctioneer_pda.to_account_info(),
        ],
    )?;
    // Perform execute_sale_cpi

    Ok(())
}
