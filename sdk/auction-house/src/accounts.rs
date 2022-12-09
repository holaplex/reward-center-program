use anchor_client::solana_sdk::pubkey::Pubkey;

pub struct DelegateAuctioneerAccounts {
    pub auction_house: Pubkey,
    pub authority: Pubkey,
    pub auctioneer_authority: Pubkey,
}
pub struct CreateAuctionHouseAccounts {
    pub treasury_mint: Pubkey,
    pub payer: Pubkey,
    pub authority: Pubkey,
    pub fee_withdrawal_destination: Pubkey,
    pub treasury_withdrawal_destination: Pubkey,
    pub treasury_withdrawal_destination_owner: Pubkey,
}

pub struct WithdrawFromTreasuryAccounts {
    pub treasury_mint: Pubkey,
    pub authority: Pubkey,
    pub treasury_withdrawal_destination: Pubkey,
    pub auction_house: Pubkey,
}
