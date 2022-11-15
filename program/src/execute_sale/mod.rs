use crate::constants::{LISTING, OFFER, REWARD_CENTER};
use crate::errors::RewardCenterError;
use crate::metaplex_cpi::auction_house::{make_auctioneer_instruction, AuctioneerInstructionArgs};
use crate::state::{Listing, Offer, RewardCenter};
use anchor_lang::{prelude::*, InstructionData};
use anchor_spl::token::{transfer, Transfer};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};
use mpl_auction_house::{
    constants::{AUCTIONEER, FEE_PAYER, PREFIX, SIGNER, TREASURY},
    cpi::accounts::AuctioneerExecuteSale,
    instruction::AuctioneerExecuteSale as AuctioneerExecuteSaleParams,
    program::AuctionHouse as AuctionHouseProgram,
    utils::assert_metadata_valid,
    AuctionHouse, Auctioneer,
};
use solana_program::program::invoke_signed;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct ExecuteSaleParams {
    pub escrow_payment_bump: u8,
    pub free_trade_state_bump: u8,
    pub seller_trade_state_bump: u8,
    pub program_as_signer_bump: u8,
}

#[derive(Accounts, Clone)]
#[instruction(execute_sale_params: ExecuteSaleParams)]
pub struct ExecuteSale<'info> {
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

pub struct ExecuteSaleIXInput<'info> {
    seller_listing_price: u64,
    seller_listing_token_size: u64,
    auction_house: &'info Box<Account<'info, AuctionHouse>>,
    reward_center: &'info Box<Account<'info, RewardCenter>>,
    metadata: &'info UncheckedAccount<'info>,
    token_account: &'info Box<Account<'info, TokenAccount>>,
    reward_center_reward_token_account: &'info Box<Account<'info, TokenAccount>>,
    buyer_reward_token_account: &'info Box<Account<'info, TokenAccount>>,
    seller_reward_token_account: &'info Box<Account<'info, TokenAccount>>,

    // Other account infos needed for CPI
    buyer_account_info: AccountInfo<'info>,
    seller_account_info: AccountInfo<'info>,
    ah_auctioneer_pda_account_info: AccountInfo<'info>,
    auction_house_account_info: AccountInfo<'info>,
    auction_house_fee_account_info: AccountInfo<'info>,
    auction_house_treasury_account_info: AccountInfo<'info>,
    buyer_receipt_token_account_info: AccountInfo<'info>,
    seller_payment_receipt_account_info: AccountInfo<'info>,
    buyer_trade_state_account_info: AccountInfo<'info>,
    free_trade_state_account_info: AccountInfo<'info>,
    seller_trade_state_account_info: AccountInfo<'info>,
    escrow_payment_account_account_info: AccountInfo<'info>,
    program_as_signer_account_info: AccountInfo<'info>,
    authority_account_info: AccountInfo<'info>,
    token_mint_account_info: AccountInfo<'info>,
    treasury_mint_account_info: AccountInfo<'info>,

    // Program accounts
    system_program_account_info: AccountInfo<'info>,
    token_program_account_info: AccountInfo<'info>,
    ata_program_account_info: AccountInfo<'info>,
    rent_account_info: AccountInfo<'info>,
}

/// `execute_sale_ix` is an inner instruction helper that takes in all the needed of inputs and parameters,
/// allowing internal calls to `execute_sale` to be made.
pub fn inner_ix<'info>(
    ExecuteSaleIXInput {
        seller_listing_price: price,
        seller_listing_token_size: token_size,
        auction_house,
        reward_center,
        reward_center_reward_token_account,
        buyer_reward_token_account,
        seller_reward_token_account,
        metadata,
        token_account,

        // Account infos,
        buyer_account_info,
        seller_account_info,
        ah_auctioneer_pda_account_info,
        auction_house_account_info,
        auction_house_fee_account_info,
        auction_house_treasury_account_info,
        buyer_receipt_token_account_info,
        seller_payment_receipt_account_info,
        buyer_trade_state_account_info,
        free_trade_state_account_info,
        seller_trade_state_account_info,
        escrow_payment_account_account_info,
        program_as_signer_account_info,
        authority_account_info,
        token_mint_account_info,
        treasury_mint_account_info,

        // Program infos
        system_program_account_info,
        token_program_account_info,
        ata_program_account_info,
        rent_account_info,
        ..
    }: ExecuteSaleIXInput<'info>,
    ExecuteSaleParams {
        escrow_payment_bump,
        free_trade_state_bump,
        program_as_signer_bump,
        ..
    }: ExecuteSaleParams,
) -> Result<()> {
    msg!("execute_sale_ix");
    let reward_center_account_info = reward_center.to_account_info();

    let auction_house_key = auction_house.key();

    let reward_center_signer_seeds: &[&[&[u8]]] = &[&[
        REWARD_CENTER.as_bytes(),
        auction_house_key.as_ref(),
        &[reward_center.bump],
    ]];

    assert_metadata_valid(metadata, token_account)?;

    // Auction house CPI
    let execute_sale_ctx_accounts = AuctioneerExecuteSale {
        buyer: buyer_account_info,
        seller: seller_account_info,
        token_account: token_account.to_account_info(),
        ah_auctioneer_pda: ah_auctioneer_pda_account_info,
        auction_house: auction_house_account_info,
        auction_house_fee_account: auction_house_fee_account_info,
        auction_house_treasury: auction_house_treasury_account_info,
        buyer_receipt_token_account: buyer_receipt_token_account_info,
        seller_payment_receipt_account: seller_payment_receipt_account_info,
        buyer_trade_state: buyer_trade_state_account_info,
        free_trade_state: free_trade_state_account_info,
        seller_trade_state: seller_trade_state_account_info,
        escrow_payment_account: escrow_payment_account_account_info,
        program_as_signer: program_as_signer_account_info,
        authority: authority_account_info,
        metadata: metadata.to_account_info(),
        token_mint: token_mint_account_info,
        treasury_mint: treasury_mint_account_info,
        auctioneer_authority: reward_center_account_info,
        system_program: system_program_account_info,
        token_program: token_program_account_info,
        ata_program: ata_program_account_info,
        rent: rent_account_info,
    };

    let execute_sale_params = AuctioneerExecuteSaleParams {
        escrow_payment_bump,
        program_as_signer_bump,
        token_size,
        buyer_price: price,
        _free_trade_state_bump: free_trade_state_bump,
    };

    let (execute_sale_ix, execute_sale_account_infos) =
        make_auctioneer_instruction(AuctioneerInstructionArgs {
            accounts: execute_sale_ctx_accounts,
            instruction_data: execute_sale_params.data(),
            auctioneer_authority: reward_center.key(),
        });

    invoke_signed(
        &execute_sale_ix,
        &execute_sale_account_infos,
        reward_center_signer_seeds,
    )?;

    let (seller_payout, buyer_payout) = reward_center.payouts(price)?;

    // Buyer transfer
    let reward_center_reward_token_balance = reward_center_reward_token_account.amount;
    if reward_center_reward_token_balance >= buyer_payout {
        transfer(
            CpiContext::new_with_signer(
                token_program_account_info,
                Transfer {
                    authority: reward_center.to_account_info(),
                    from: reward_center_reward_token_account.to_account_info(),
                    to: buyer_reward_token_account.to_account_info(),
                },
                reward_center_signer_seeds,
            ),
            buyer_payout,
        )?;
    }

    // Seller transfer
    reward_center_reward_token_account.reload()?;
    let reward_center_reward_token_balance = reward_center_reward_token_account.amount;
    if reward_center_reward_token_balance >= seller_payout {
        transfer(
            CpiContext::new_with_signer(
                token_program_account_info,
                Transfer {
                    authority: reward_center.to_account_info(),
                    from: reward_center_reward_token_account.to_account_info(),
                    to: seller_reward_token_account.to_account_info(),
                },
                reward_center_signer_seeds,
            ),
            seller_payout,
        )?
    };

    Ok(())
}

pub fn handler(ctx: Context<ExecuteSale>, params: ExecuteSaleParams) -> Result<()> {
    let seller_listing = &ctx.accounts.listing;

    let auction_house = &ctx.accounts.auction_house;
    let reward_center = &ctx.accounts.reward_center;
    let metadata = &ctx.accounts.metadata;
    let token_account = &ctx.accounts.token_account;
    let reward_center_reward_token_account = &ctx.accounts.reward_center_reward_token_account;
    let buyer_reward_token_account = &ctx.accounts.buyer_reward_token_account;
    let seller_reward_token_account = &ctx.accounts.seller_reward_token_account;

    let price = seller_listing.price;
    let token_size = seller_listing.token_size;

    inner_ix(
        ExecuteSaleIXInput {
            seller_listing_price: seller_listing.price,
            seller_listing_token_size: seller_listing.token_size,
            auction_house,
            reward_center,
            metadata,
            token_account,
            reward_center_reward_token_account,
            buyer_reward_token_account,
            seller_reward_token_account,

            // Account infos for CPI
            buyer_account_info: ctx.accounts.buyer.to_account_info(),
            seller_account_info: ctx.accounts.seller.to_account_info(),
            ah_auctioneer_pda_account_info: ctx.accounts.ah_auctioneer_pda.to_account_info(),
            auction_house_account_info: ctx.accounts.auction_house.to_account_info(),
            auction_house_fee_account_info: ctx
                .accounts
                .auction_house_fee_account
                .to_account_info(),
            auction_house_treasury_account_info: ctx
                .accounts
                .auction_house_treasury
                .to_account_info(),
            buyer_receipt_token_account_info: ctx
                .accounts
                .buyer_receipt_token_account
                .to_account_info(),
            seller_payment_receipt_account_info: ctx
                .accounts
                .seller_payment_receipt_account
                .to_account_info(),
            buyer_trade_state_account_info: ctx.accounts.buyer_trade_state.to_account_info(),
            free_trade_state_account_info: ctx.accounts.free_seller_trade_state.to_account_info(),
            seller_trade_state_account_info: ctx.accounts.seller_trade_state.to_account_info(),
            escrow_payment_account_account_info: ctx
                .accounts
                .escrow_payment_account
                .to_account_info(),
            program_as_signer_account_info: ctx.accounts.program_as_signer.to_account_info(),
            authority_account_info: ctx.accounts.authority.to_account_info(),
            token_mint_account_info: ctx.accounts.token_mint.to_account_info(),
            treasury_mint_account_info: ctx.accounts.treasury_mint.to_account_info(),

            // Program accounts
            system_program_account_info: ctx.accounts.system_program.to_account_info(),
            token_program_account_info: ctx.accounts.token_program.to_account_info(),
            ata_program_account_info: ctx.accounts.ata_program.to_account_info(),
            rent_account_info: ctx.accounts.rent.to_account_info(),
        },
        params,
    )?;
    Ok(())
}
