use std::{
    fs::{read_to_string, File},
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Context, Result};
use dirs::config_dir;
use log::debug;
use serde::{Deserialize, Serialize};
use solana_sdk::signature::Keypair;

#[derive(Debug, Deserialize, Serialize)]
pub struct SolanaConfiguration {
    pub json_rpc_url: String,
    pub keypair_path: String,
    pub commitment: String,
}

/// # Errors
///
/// Will return `Err` if config dir mismatches and/or the config path is unable to open and/or failed to parse the config file
pub fn parse_solana_configuration() -> Result<Option<SolanaConfiguration>> {
    let mut config_path = config_dir().ok_or_else(|| anyhow!("Platform is not supported"))?;
    config_path.extend(["solana", "cli", "config.yml"]);

    let conf_file = match File::open(config_path) {
        Ok(f) => f,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(err) => return Err(err).context("Failed to open config file"),
    };

    serde_yaml::from_reader(&conf_file).context("Failed to parse config")
}

/// # Errors
///
/// Will return `Err` if keypair path is incorrect or failed to parse it correctl
pub fn parse_keypair(
    keypair_opt: &Option<PathBuf>,
    sol_config_option: &Option<SolanaConfiguration>,
) -> Result<Keypair> {
    match (keypair_opt, sol_config_option) {
        (Some(keypair_path), _) => {
            read_keypair(keypair_path).context("Failed to read keypair file.")
        },
        (None, Some(ref sol_config)) => {
            read_keypair(&sol_config.keypair_path).context("Failed to read keypair file.")
        },
        (None, None) => {
            let mut id_path = config_dir().ok_or_else(|| anyhow!("Platform is not supported"))?;
            id_path.extend(["solana", "id.json"]);

            read_keypair(&id_path).context("Failed to read keypair file.")
        },
    }
}

/// # Errors
///
/// Will return `Err` if `path` does not exist or the keypair is to unable to parse
pub fn read_keypair<P: AsRef<Path>>(path: P) -> Result<Keypair> {
    let secret_string = read_to_string(path).context("Can't find key file")?;

    // Try to decode the secret string as a JSON array of ints first and then as a base58 encoded string to support Phantom private keys.

    let secret_bytes = serde_json::from_str(&secret_string)
        .map_err(|e| debug!("Failed to parse keypair as JSON: {}", e))
        .or_else(|()| {
            bs58::decode(&secret_string.trim())
                .into_vec()
                .map_err(|e| debug!("Failed to parse keypair as base58: {}", e))
        })
        .map_err(|()| anyhow!("Unsupported key type!"))?;

    let keypair = Keypair::from_bytes(&secret_bytes)?;
    Ok(keypair)
}
