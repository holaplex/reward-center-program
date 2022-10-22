use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum PayoutOperation {
    Multiple,
    Divide,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct CreateRewardCenterParams {
    pub mathematical_operand: PayoutOperation,
    pub seller_reward_payout_basis_points: u16,
    pub payout_numeral: u16,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct EditRewardCenterParams {
    pub mathematical_operand: PayoutOperation,
    pub seller_reward_payout_basis_points: u16,
    pub payout_numeral: u16,
}
