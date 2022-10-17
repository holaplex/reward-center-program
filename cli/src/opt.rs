use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "Reward Center",
    about = "A Metaplex auctioneer program that distributes spl token to the buyer and seller of NFTs"
)]
pub struct RewardCenterOption {
    /// RPC endpoint url to override using the Solana config or the hard-coded default
    #[structopt(short, long, global = true)]
    pub rpc: Option<String>,

    /// Timeout to override default value of 90 seconds
    #[structopt(short = "T", long, global = true, default_value = "90")]
    pub timeout: u64,

    /// Log level
    #[structopt(short, long, global = true, default_value = "off")]
    pub log_level: String,

    /// All available commands
    #[structopt(subcommand)]
    pub cmd: RewardCenterCommands,
}

#[derive(Debug, StructOpt)]
pub enum RewardCenterCommands {
    /// Create Reward Center
    #[structopt(name = "create")]
    Create {
        /// Reward center config file path
        #[structopt(short, long, default_value = "src/json/reward_center.json")]
        config_file: String,

        /// Optional Auction House address
        #[structopt(short, long)]
        auction_house: Option<String>,

        /// Optional Rewards mint address
        #[structopt(short, long)]
        rewards_mint: Option<String>,
    },
}
