use anchor_lang::prelude::*;

#[event]
pub struct RewardCenterTreasuryWithdrawn {
    pub rewards_mint: Pubkey,
    pub reward_center_authority: Pubkey,
    pub destination_reward_token_account: Pubkey,
    pub withdrawal_amount: u64,
}
