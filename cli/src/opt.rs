use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "Reward Center",
    about = "A Metaplex auctioneer program that distributes spl token to the buyer and seller of NFTs"
)]
pub struct Opt {
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
    pub cmd: Command,
}

#[derive(Debug, StructOpt)]
pub enum Command {
    /// Create Reward Center
    #[structopt(name = "create")]
    Create {
        /// Reward center config file path
        #[structopt(short, long, default_value = "src/json/reward_center.json")]
        config_file: String,

        /// Optional Auction House address
        #[structopt(short = "a", long)]
        auction_house: Option<String>,

        /// Optional Rewards mint address
        #[structopt(short = "M", long)]
        mint_rewards: Option<String>,

        /// Path to the creator's keypair file (mint auth keypair if required to create)
        #[structopt(short, long)]
        keypair: Option<String>,
    },

    /// Edit reward center's reward rules
    #[structopt(name = "edit")]
    Edit {
        /// Reward center address
        #[structopt(short = "R", long)]
        reward_center: String,

        /// Auction house address
        #[structopt(short, long)]
        auction_house: String,

        /// Reward center config file path
        #[structopt(short, long, default_value = "src/json/reward_center.json")]
        config_file: String,

        /// Path to the creator's keypair file (mint auth keypair if required to create)
        #[structopt(short, long)]
        keypair: Option<String>,
    },

    /// Fund reward center
    #[structopt(name = "fund")]
    Fund {
        /// Reward center address
        #[structopt(short = "R", long)]
        reward_center: String,

        /// Path to the creator's keypair file (mint auth keypair if required to create)
        #[structopt(short, long)]
        keypair: Option<String>,

        /// Funding amount
        #[structopt(short, long)]
        amount: u64,
    },
}
