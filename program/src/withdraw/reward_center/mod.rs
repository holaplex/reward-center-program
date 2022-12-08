use anchor_lang::{prelude::*, AnchorDeserialize};
use anchor_spl::token::{transfer, Token, TokenAccount, Transfer};

use mpl_auction_house::{constants::PREFIX, AuctionHouse};
use solana_program::program_pack::IsInitialized;

use crate::{
    constants::REWARD_CENTER, errors::RewardCenterError, events::RewardCenterTreasuryWithdrawn,
    state::RewardCenter,
};

#[derive(AnchorDeserialize, AnchorSerialize)]
pub struct WithdrawRewardCenterFundsParams {
    pub withdrawal_amount: u64,
}

/// Accounts for the [`withdraw_reward_center_funds` handler](reward_center/fn.withdraw_reward_center_funds.html).
#[derive(Accounts, Clone)]
#[instruction(withdraw_reward_center_funds_params: WithdrawRewardCenterFundsParams)]
pub struct WithdrawRewardCenterFunds<'info> {
    /// User wallet account.
    #[
      account(
        mut,
        constraint = wallet.key() == auction_house.authority @ RewardCenterError::SignerNotAuthorized
      )
    ]
    pub wallet: Signer<'info>,

    // Reward center reward token account
    #[account(
        mut,
        constraint = reward_center_reward_token_account.mint == reward_center.token_mint @ RewardCenterError::MintMismatch,
        constraint = reward_center_reward_token_account.owner == reward_center.key() @ RewardCenterError::TokenOwnerMismatch,
        constraint = reward_center_reward_token_account.amount >= withdraw_reward_center_funds_params.withdrawal_amount @ RewardCenterError::InsufficientFunds,
    )]
    pub reward_center_reward_token_account: Account<'info, TokenAccount>,

    // Destination reward token account where the rewards get transferred
    #[account(
        mut,
        constraint = destination_reward_token_account.mint == reward_center.token_mint @ RewardCenterError::MintMismatch,
        constraint = destination_reward_token_account.owner == wallet.key() @ RewardCenterError::TokenOwnerMismatch,
        constraint = destination_reward_token_account.is_initialized() @
        ProgramError::UninitializedAccount
    )]
    pub destination_reward_token_account: Account<'info, TokenAccount>,

    /// The auctioneer program PDA running this auction.
    #[account(
        seeds = [REWARD_CENTER.as_bytes(), auction_house.key().as_ref()],
        bump
    )]
    pub reward_center: Account<'info, RewardCenter>,

    /// Auction House instance PDA account.
    #[account(
        seeds = [
            PREFIX.as_bytes(),
            auction_house.creator.as_ref(),
            auction_house.treasury_mint.as_ref()
        ],
        seeds::program = mpl_auction_house::id(),
        bump = auction_house.bump
    )]
    pub auction_house: Box<Account<'info, AuctionHouse>>,

    pub token_program: Program<'info, Token>,
}

pub fn handler(
    ctx: Context<WithdrawRewardCenterFunds>,
    WithdrawRewardCenterFundsParams { withdrawal_amount }: WithdrawRewardCenterFundsParams,
) -> Result<()> {
    let reward_center = &ctx.accounts.reward_center;
    let reward_center_bump = ctx.accounts.reward_center.bump;
    let reward_center_reward_token_account = &ctx.accounts.reward_center_reward_token_account;
    let destination_reward_token_account = &ctx.accounts.destination_reward_token_account;

    let auction_house = &ctx.accounts.auction_house;
    let auction_house_key = auction_house.key();

    let token_program = &ctx.accounts.token_program;

    let reward_center_signer_seeds: &[&[&[u8]]] = &[&[
        REWARD_CENTER.as_bytes(),
        auction_house_key.as_ref(),
        &[reward_center_bump],
    ]];

    let token_transfer_ctx = CpiContext::new_with_signer(
        token_program.to_account_info(),
        Transfer {
            from: reward_center_reward_token_account.to_account_info(),
            to: destination_reward_token_account.to_account_info(),
            authority: reward_center.to_account_info(),
        },
        reward_center_signer_seeds,
    );

    transfer(token_transfer_ctx, withdrawal_amount)?;

    emit!(RewardCenterTreasuryWithdrawn {
        reward_center_authority: ctx.accounts.wallet.key(),
        destination_reward_token_account: destination_reward_token_account.key(),
        rewards_mint: ctx.accounts.reward_center.token_mint.key(),
        withdrawal_amount: withdrawal_amount
    });

    Ok(())
}
