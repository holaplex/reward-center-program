use crate::constants::{OFFER, REWARD_CENTER};
use crate::errors::RewardCenterError;
use crate::metaplex_cpi::auction_house::{make_auctioneer_instruction, AuctioneerInstructionArgs};
use crate::state::{Offer, RewardCenter};
use anchor_lang::{prelude::*, InstructionData};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{transfer, Mint, Token, TokenAccount, Transfer},
};
use mpl_auction_house::constants::TREASURY;
use mpl_auction_house::{
    constants::{AUCTIONEER, FEE_PAYER, PREFIX, SIGNER},
    cpi::accounts::{AuctioneerExecuteSale, AuctioneerSell},
    instruction::AuctioneerExecuteSale as AuctioneerExecuteSaleParams,
    instruction::AuctioneerSell as AuctioneerSellParams,
    program::AuctionHouse as AuctionHouseProgram,
    utils::assert_metadata_valid,
    AuctionHouse, Auctioneer,
};
use solana_program::program::invoke_signed;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct AcceptOfferParams {
    pub escrow_payment_bump: u8,
    pub free_trade_state_bump: u8,
    pub program_as_signer_bump: u8,
    pub seller_trade_state_bump: u8,
    pub buyer_trade_state_bump: u8,
}

#[derive(Accounts, Clone)]
#[instruction(accept_offer_params: AcceptOfferParams)]
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
        // Transitive equality check enforced by check on buyer_reward_token_account
        constraint = buyer_reward_token_account.mint == seller_reward_token_account.mint @ RewardCenterError::MintMismatch,
        constraint = seller_reward_token_account.owner == seller.key() @ RewardCenterError::SellerTokenAccountMismatch,
    )]
    pub seller_reward_token_account: Box<Account<'info, TokenAccount>>,

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
        bump = accept_offer_params.escrow_payment_bump
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

    /// CHECK: Not dangerous. Account seeds checked in constraint.
    /// Buyer trade state PDA account encoding the buy order.
    #[account(
        mut,
        seeds = [
            PREFIX.as_bytes(),
            buyer.key().as_ref(),
            auction_house.key().as_ref(),
            treasury_mint.key().as_ref(),
            token_account.mint.as_ref(),
            offer.price.to_le_bytes().as_ref(),
            offer.token_size.to_le_bytes().as_ref()
        ],
        seeds::program = auction_house_program,
        bump = accept_offer_params.buyer_trade_state_bump
    )]
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
            &offer.token_size.to_le_bytes()
        ],
        seeds::program = auction_house_program,
        bump = accept_offer_params.seller_trade_state_bump,
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
            &offer.token_size.to_le_bytes()
        ],
        seeds::program = auction_house_program,
        bump = accept_offer_params.free_trade_state_bump
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
        bump = accept_offer_params.program_as_signer_bump
    )]
    pub program_as_signer: UncheckedAccount<'info>,

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

pub fn handler<'info>(
    ctx: Context<'_, '_, '_, 'info, AcceptOffer<'info>>,
    AcceptOfferParams {
        seller_trade_state_bump,
        free_trade_state_bump,
        program_as_signer_bump,
        escrow_payment_bump,
        ..
    }: AcceptOfferParams,
) -> Result<()> {
    let auction_house_key = ctx.accounts.auction_house.key();
    let reward_center_bump = ctx.accounts.reward_center.bump;
    let offer = &ctx.accounts.offer;
    let reward_center = &ctx.accounts.reward_center;
    let token_account = &ctx.accounts.token_account;
    let metadata = &ctx.accounts.metadata;
    let token_size = offer.token_size;
    let buyer_price = offer.price;

    assert_metadata_valid(metadata, token_account)?;

    let reward_center_signer_seeds: &[&[&[u8]]] = &[&[
        REWARD_CENTER.as_bytes(),
        auction_house_key.as_ref(),
        &[reward_center_bump],
    ]];

    let (create_listing_ix, create_listing_account_infos) =
        make_auctioneer_instruction(AuctioneerInstructionArgs {
            accounts: AuctioneerSell {
                metadata: metadata.to_account_info(),
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
            },
            instruction_data: AuctioneerSellParams {
                trade_state_bump: seller_trade_state_bump,
                free_trade_state_bump,
                program_as_signer_bump,
                token_size,
            }
            .data(),
            auctioneer_authority: ctx.accounts.reward_center.key(),
            remaining_accounts: None,
        });

    invoke_signed(
        &create_listing_ix,
        &create_listing_account_infos,
        reward_center_signer_seeds,
    )?;

    let (execute_sale_ix, execute_sale_account_infos) =
        make_auctioneer_instruction(AuctioneerInstructionArgs {
            accounts: AuctioneerExecuteSale {
                buyer: ctx.accounts.buyer.to_account_info(),
                seller: ctx.accounts.seller.to_account_info(),
                token_account: ctx.accounts.token_account.to_account_info(),
                ah_auctioneer_pda: ctx.accounts.ah_auctioneer_pda.to_account_info(),
                auction_house: ctx.accounts.auction_house.to_account_info(),
                auction_house_fee_account: ctx.accounts.auction_house_fee_account.to_account_info(),
                auction_house_treasury: ctx.accounts.auction_house_treasury.to_account_info(),
                buyer_receipt_token_account: ctx
                    .accounts
                    .buyer_receipt_token_account
                    .to_account_info(),
                seller_payment_receipt_account: ctx
                    .accounts
                    .seller_payment_receipt_account
                    .to_account_info(),
                buyer_trade_state: ctx.accounts.buyer_trade_state.to_account_info(),
                free_trade_state: ctx.accounts.free_seller_trade_state.to_account_info(),
                seller_trade_state: ctx.accounts.seller_trade_state.to_account_info(),
                escrow_payment_account: ctx.accounts.escrow_payment_account.to_account_info(),
                program_as_signer: ctx.accounts.program_as_signer.to_account_info(),
                authority: ctx.accounts.authority.to_account_info(),
                metadata: ctx.accounts.metadata.to_account_info(),
                token_mint: ctx.accounts.token_mint.to_account_info(),
                treasury_mint: ctx.accounts.treasury_mint.to_account_info(),
                auctioneer_authority: ctx.accounts.reward_center.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                token_program: ctx.accounts.token_program.to_account_info(),
                ata_program: ctx.accounts.ata_program.to_account_info(),
                rent: ctx.accounts.rent.to_account_info(),
            },
            instruction_data: AuctioneerExecuteSaleParams {
                escrow_payment_bump,
                program_as_signer_bump,
                token_size,
                buyer_price,
                _free_trade_state_bump: free_trade_state_bump,
            }
            .data(),
            auctioneer_authority: ctx.accounts.reward_center.key(),
            remaining_accounts: Some(ctx.remaining_accounts),
        });

    invoke_signed(
        &execute_sale_ix,
        &execute_sale_account_infos,
        reward_center_signer_seeds,
    )?;

    let (seller_payout, buyer_payout) = reward_center.payouts(buyer_price)?;

    // Buyer transfer
    let reward_center_reward_token_balance = ctx.accounts.reward_center_reward_token_account.amount;
    if buyer_payout > 0 && reward_center_reward_token_balance >= buyer_payout {
        transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    authority: ctx.accounts.reward_center.to_account_info(),
                    from: ctx
                        .accounts
                        .reward_center_reward_token_account
                        .to_account_info(),
                    to: ctx.accounts.buyer_reward_token_account.to_account_info(),
                },
                reward_center_signer_seeds,
            ),
            buyer_payout,
        )?;
    }

    // Seller transfer
    ctx.accounts.reward_center_reward_token_account.reload()?;
    let reward_center_reward_token_balance = ctx.accounts.reward_center_reward_token_account.amount;
    if seller_payout > 0 && reward_center_reward_token_balance >= seller_payout {
        transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    authority: ctx.accounts.reward_center.to_account_info(),
                    from: ctx
                        .accounts
                        .reward_center_reward_token_account
                        .to_account_info(),
                    to: ctx.accounts.seller_reward_token_account.to_account_info(),
                },
                reward_center_signer_seeds,
            ),
            seller_payout,
        )?
    };

    Ok(())
}
