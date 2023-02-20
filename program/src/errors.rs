use anchor_lang::prelude::*;

#[error_code]
pub enum RewardCenterError {
    // 6000
    #[msg("Bump seed not in hash map")]
    BumpSeedNotInHashMap,

    // 6001
    #[msg("Unauthorized signer")]
    SignerNotAuthorized,

    // 6002
    #[msg("Math numerical overflow")]
    NumericalOverflowError,

    // 6003
    #[msg("The mints do not match")]
    MintMismatch,

    // 6004
    #[msg("Listing and offer prices do not match")]
    PriceMismatch,

    // 6005
    #[msg("Buyer token account owner does not match the buyer")]
    BuyerTokenAccountMismatch,

    // 6006
    #[msg("Seller token account owner does not match the seller")]
    SellerTokenAccountMismatch,

    // 6007
    #[msg(
        "The number of decimals for auction house treasury mint do not match reward mint decimals"
    )]
    RewardMintDecimalMismatch,

    // 6008
    #[msg("The treasury does not match the one present on the auction house")]
    AuctionHouseTreasuryMismatch,

    // 6009
    #[msg("The account address bumps do not match")]
    BumpMismatch,

    // 6010
    #[msg("The given token account owner does not match")]
    TokenOwnerMismatch,

    // 6011
    #[msg("The given token account has insufficient funds")]
    InsufficientFunds,

    // 6012
    #[msg("The listing price cannot be zero")]
    PriceInvalid,
}
