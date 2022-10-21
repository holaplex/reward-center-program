use anchor_lang::prelude::*;

#[error_code]
pub enum ListingRewardsError {
    // 6000
    #[msg("Bump seed not in hash map")]
    BumpSeedNotInHashMap,

    // 6001
    #[msg("Unauthorized signer")]
    SignerNotAuthorized,

    // 6005
    #[msg("The seller doesnt match the provided wallet")]
    SellerWalletMismatch,

    // 6006
    #[msg("The rewards were already claimed for this listing")]
    RewardsAlreadyClaimed,

    // 6007
    #[msg("The listings is not eligible for rewards yet")]
    IneligibaleForRewards,

    // 6008
    #[msg("Math numerical overflow")]
    NumericalOverflowError,

    // 6009
    #[msg("The mints do not match")]
    MintMismatch,

    // 6010
    #[msg("Listing and offer prices do not match")]
    PriceMismatch,

    // 6009
    #[msg("Buyer token account owner does not match the buyer")]
    BuyerTokenAccountMismatch,

    // 6010
    #[msg("Seller token account owner does not match the seller")]
    SellerTokenAccountMismatch,
}
