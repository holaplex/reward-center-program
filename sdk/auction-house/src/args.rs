use mpl_auction_house::AuthorityScope;

pub struct CreateAuctionHouseData {
    pub seller_fee_basis_points: u16,
    pub requires_sign_off: bool,
    pub can_change_sale_price: bool,
}

pub struct DelegateAuctioneerData {
    pub scopes: Vec<AuthorityScope>,
}
