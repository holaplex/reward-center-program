use lazy_static::lazy_static;
use std::{collections::HashMap, sync::RwLock};

pub const PUBLIC_RPC_URLS: &[&str] = &[
    "https://api.devnet.solana.com",
    "https://api.testnet.solana.com",
    "https://api.mainnet-beta.solana.com",
    "https://solana-api.projectserum.com",
];

pub const DEFAULT_RPC_DELAY_MS: u32 = 200;

lazy_static! {
    pub static ref USE_RATE_LIMIT: RwLock<bool> = RwLock::new(false);
    pub static ref RPC_DELAY_NS: RwLock<u32> = RwLock::new(DEFAULT_RPC_DELAY_MS * 1_000_000);
    pub static ref RATE_LIMIT_DELAYS: HashMap<&'static str, u32> =
        [("https://ssc-dao.genesysgo.net", 25),]
            .iter()
            .copied()
            .collect();
}
