pub mod fixtures;

use solana_program_test::*;
use solana_sdk::{account::Account, pubkey::Pubkey};

pub const TEN_SOL: u64 = 10_000_000_000;
pub const ONE_SOL: u64 = 1_000_000_000;
pub const TEST_COLLECTION: &str = "Cehzo7ugAvuYcTst9HF24ackLxnrpDkzHFajj17FuyUR";

pub fn setup_program<'a>() -> ProgramTest {
    let mut program = ProgramTest::new("hpl_reward_center", hpl_reward_center::id(), None);
    program.add_program("mpl_auction_house", mpl_auction_house::id(), None);
    program.add_program("mpl_token_metadata", mpl_token_metadata::id(), None);
    program.add_program("mpl_token_auth_rules", mpl_token_auth_rules::id(), None);
    program.set_compute_max_units(u64::MAX);

    program
}

pub async fn get_account(bank_client: &mut BanksClient, address: Pubkey) -> Option<Account> {
    bank_client.get_account(address).await.unwrap()
}
