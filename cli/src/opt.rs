use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    name = "Reward Center",
    about = "A Metaplex auctioneer program that distributes spl token to the buyer and seller of NFTs"
)]
pub struct Opt {
    /// RPC endpoint url to override using the Solana config or the hard-coded default
    #[arg(short, long, global = true)]
    pub rpc: Option<String>,

    /// Timeout to override default value of 90 seconds
    #[arg(short = 'T', long, global = true, default_value = "90")]
    pub timeout: u64,

    /// All available commands
    #[command(subcommand)]
    pub cmd: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Create Reward Center
    #[clap(name = "create")]
    Create {
        /// Reward center config file path
        #[arg(short, long, default_value = "src/json/reward_center.json")]
        config_file: PathBuf,

        /// Optional Auction House address
        #[arg(short = 'a', long)]
        auction_house: Option<String>,

        /// Optional Rewards mint address
        #[arg(short = 'M', long)]
        mint_rewards: Option<String>,

        /// Path to the creator's keypair file (mint auth keypair if required to create)
        #[arg(short, long)]
        keypair: Option<PathBuf>,
    },

    /// Create address lookup table
    #[clap(name = "create-alt")]
    CreateAddressTable {
        /// Optional Auction House address
        #[arg(short, long)]
        auction_house: String,

        /// Path to the address look up table's authority keypair file
        #[arg(short, long)]
        keypair: Option<PathBuf>,
    },

    /// Edit reward center's reward rules
    #[clap(name = "edit")]
    Edit {
        /// Reward center address
        #[arg(short = 'R', long)]
        reward_center: String,

        /// Auction house address
        #[arg(short, long)]
        auction_house: String,

        /// Reward center config file path
        #[arg(short, long, default_value = "src/json/reward_center.json")]
        config_file: PathBuf,

        /// Path to the reward center authority's keypair file
        #[arg(short, long)]
        keypair: Option<PathBuf>,
    },

    /// Fund reward center
    #[clap(name = "fund")]
    Fund {
        /// Reward center address
        #[arg(short = 'R', long)]
        reward_center: String,

        /// Path to the reward center authority keypair file
        #[arg(short, long)]
        keypair: Option<PathBuf>,

        /// Funding amount (excluding decimals)
        #[arg(short, long)]
        amount: u64,
    },

    /// Fetch Treasury Balance
    #[clap(name = "balance")]
    FetchTreasuryBalance {
        /// Reward center address
        #[arg(short = 'R', long)]
        reward_center: String,

        /// Path to the reward center authority keypair file
        #[arg(short, long)]
        keypair: Option<PathBuf>,
    },

    /// Fetch Reward Center State details
    #[clap(name = "show")]
    FetchRewardCenterState {
        /// Reward center address
        #[arg(short = 'R', long)]
        reward_center: String,

        /// Path to the reward center authority keypair file
        #[arg(short, long)]
        keypair: Option<PathBuf>,
    },

    /// Withdraw from Reward center treasury
    #[clap(name = "withdraw-reward-center")]
    WithdrawRewardCenter {
        /// Reward center address
        #[arg(short = 'R', long)]
        reward_center: String,

        /// Path to the reward center authority keypair file
        #[arg(short, long)]
        keypair: Option<PathBuf>,

        /// Amount to withdraw (excluding decimals)
        #[arg(short = 'a', long)]
        amount: u64,
    },

    /// Withdraw from Auction House treasury
    #[clap(name = "withdraw-auction-house")]
    WithdrawAuctionHouse {
        /// Auction house address
        #[arg(short = 'A', long)]
        auction_house: String,

        /// Path to the reward center authority keypair file
        #[arg(short, long)]
        keypair: Option<PathBuf>,

        /// Amount to withdraw (excluding decimals)
        #[arg(short = 'a', long)]
        amount: u64,
    },
}
