use std::{
    env,
    fs::{read_to_string, File},
    path::Path,
};

use anyhow::{anyhow, Context, Result};

use serde::{Deserialize, Serialize};
use serde_yaml;
use solana_sdk::signature::Keypair;

#[derive(Debug, Deserialize, Serialize)]
pub struct SolanaConfig {
    pub json_rpc_url: String,
    pub keypair_path: String,
    pub commitment: String,
}

pub fn parse_solana_config() -> Option<SolanaConfig> {
    let home = if cfg!(unix) {
        env::var_os("HOME").expect("Coulnd't find UNIX home key.")
    } else if cfg!(windows) {
        let drive = env::var_os("HOMEDRIVE").expect("Coulnd't find Windows home drive key.");
        let path = env::var_os("HOMEPATH").expect("Coulnd't find Windows home path key.");
        Path::new(&drive).join(&path).as_os_str().to_owned()
    } else if cfg!(target_os = "macos") {
        env::var_os("HOME").expect("Coulnd't find MacOS home key.")
    } else {
        panic!("Unsupported OS!");
    };

    let config_path = Path::new(&home)
        .join(".config")
        .join("solana")
        .join("cli")
        .join("config.yml");

    let conf_file = match File::open(config_path) {
        Ok(f) => f,
        Err(_) => return None,
    };
    serde_yaml::from_reader(&conf_file).ok()
}

pub fn parse_keypair(
    keypair_opt: &Option<String>,
    sol_config_option: &Option<SolanaConfig>,
) -> Keypair {
    let keypair = match keypair_opt {
        Some(keypair_path) => read_keypair(&keypair_path).expect("Failed to read keypair file."),
        None => match sol_config_option {
            Some(ref sol_config) => {
                read_keypair(&sol_config.keypair_path).expect("Failed to read keypair file.")
            }
            None => read_keypair(&(*shellexpand::tilde("~/.config/solana/id.json")).to_string())
                .expect("Failed to read keypair file."),
        },
    };
    keypair
}

pub fn read_keypair(path: &String) -> Result<Keypair> {
    let secret_string: String = read_to_string(path).context("Can't find key file")?;

    // Try to decode the secret string as a JSON array of ints first and then as a base58 encoded string to support Phantom private keys.
    let secret_bytes: Vec<u8> = match serde_json::from_str(&secret_string) {
        Ok(bytes) => bytes,
        Err(_) => match bs58::decode(&secret_string.trim()).into_vec() {
            Ok(bytes) => bytes,
            Err(_) => return Err(anyhow!("Unsupported key type!")),
        },
    };

    let keypair = Keypair::from_bytes(&secret_bytes)?;
    Ok(keypair)
}
