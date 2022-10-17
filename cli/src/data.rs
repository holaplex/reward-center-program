use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum PayoutOperation {
    Multiple,
    Divide,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateRewardCenterCofnig {
    pub mathematical_operand: PayoutOperation,
    pub seller_reward_payout_basis_points: u16,
    pub payout_numeral: u16,
}
