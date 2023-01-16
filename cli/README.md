[![Crate](https://img.shields.io/crates/v/hpl-reward-center)](https://crates.io/crates/hpl-reward-center)
[![Downloads](https://img.shields.io/crates/d/hpl-reward-center)](https://crates.io/crates/hpl-reward-center)
[![Build Status](https://img.shields.io/github/workflow/status/holaplex/reward-center-program/CI)](https://github.com/holaplex/reward-center-program/actions)
[![License](https://img.shields.io/crates/l/hpl-reward-center)](https://github.com/holaplex/reward-center-program/blob/main/LICENSE)

# Reward Center CLI

## Overview

A Metaplex auctioneer program that distributes spl token to the buyer and seller of NFTs.

## Installation

### Install Binary
Copy the following to a terminal:

```bash
bash <(curl -sSf https://github.com/holaplex/reward-center-program/tree/main/cli/scripts/install.sh)
```

If you get errors you may need dependencies:

Ubuntu:

```bash
sudo apt install libssl-dev libudev-dev pkg-config
```

MacOS may need openssl:

```bash
brew install openssl@3
```

### Binaries

Linux, MacOS and Windows binaries available in [releases](https://github.com/holaplex/reward-center-program/releases).

### Install From Source

Requires Rust 1.58 or later.

Install [Rust](https://www.rust-lang.org/tools/install).

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Clone the source:

```bash
git clone git@github.com:holaplex/reward-center-program.git
```

or

```bash
git clone https://github.com/holaplex/reward-center-program.git
```

Change directory and check out the `main` branch:

```bash
cd reward-center-program/cli
git checkout main
```

Install or build with Rust:

```bash
cargo install --path ./
```

or

```bash
cargo build --release
```

## Commands

### Create Reward Center
Create a reward center that also creates an auction house and rewards mint along if required

#### Usage
```sh
reward-center-cli create -c <CONFIG_FILE> -a <AUCTION_HOUSE> -M <MINT_REWARDS> -k <KEYPAIR> -r <RPC> -T <TIMEOUT>
```

### Create Address Table Lookup
Creates an address table lookup account to facilitate adding more addresses at situations when we require to pass more than 32 accounts during an offer acceptance.

#### Usage
```sh
reward-center-cli create-alt  -a <AUCTION_HOUSE> -k <KEYPAIR> -r <RPC> -T <TIMEOUT>
```

### Edit Reward Center
Allows a reward center authority to edit the reward rules configuration.

#### Usage
```sh
reward-center-cli edit -c <CONFIG_FILE> -R <REWARD_CENTER> -a <AUCTION_HOUSE> -k <KEYPAIR> -r <RPC> -T <TIMEOUT>
```

### Fund Reward Center
Allows a reward center authority to fund the reward center token account.

#### Usage
```sh
reward-center-cli fund -R <REWARD_CENTER> -a <AMOUNT> -k <KEYPAIR> -r <RPC> -T <TIMEOUT>
```

### Withdraw Reward Center
Allows a reward center authority to withdraw the reward center treasury funds.

#### Usage
```sh
reward-center-cli withdraw-reward-center -R <REWARD_CENTER> -a <AMOUNT> -k <KEYPAIR> -r <RPC> -T <TIMEOUT>
```

### Withdraw Auction House
Allows an auction house (same as reward center if created along) authority to withdraw the auction house treasury funds.

#### Usage
```sh
reward-center-cli withdraw-auction-house -A <AUCTION_HOUSE> -a <AMOUNT> -k <KEYPAIR> -r <RPC> -T <TIMEOUT>
```

### Get Reward Center treasury balance
Fetches the current treasury balance held by the reward center.

#### Usage
```sh
reward-center-cli balance -R <REWARD_CENTER> -k <KEYPAIR> -r <RPC> -T <TIMEOUT>
```

### Get Reward Center state
Fetches the current state values of a reward center.

#### Usage
```sh
reward-center-cli show -R <REWARD_CENTER> -k <KEYPAIR> -r <RPC> -T <TIMEOUT>
```

### Create an address lookup table

Generates a address lookup table for reducing the number of accounts needed to be sent with each transaction.

```sh
reward-center-cli create-alt --keypair <KEYPAIR> --rpc <RPC> --auction-house <AUCTION_HOUSE>
```
