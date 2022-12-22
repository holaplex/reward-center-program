# Creating a Reward Center

This guide takes you through setting up a reward center including a SPL token for rewards and address lookup table (ALT) for the [Holaplex Reward Center Metaplex JS Plugin](https://github.com/holaplex/reward-center-metaplex-js-plugin).

## Dependencies

- [Reward Center CLI](https://github.com/holaplex/reward-center-program/tree/main/cli)
- [Solana Tool Suite](https://docs.solana.com/cli/install-solana-cli-tools)

## SPL Token

In order to distribute rewards to buyers and sellers you need a SPL token.

1. Create a mint.

```
spl-token create
```

2. Create a token account for holding the token for the current wallet context.

```
spl-token create-account {mint}
```

3. Mint some tokens from the mint for a certain amount.

```
spl-token mint {mint} {amount}
```

## Reward Center

Now that you have a token its time to create your reward center. For the complete list of commands see the Reward Center CLI [README](https://github.com/holaplex/reward-center-program/blob/main/cli/README.md).

1. Create the reward center referencing a config for setting up the reward center.

```
# reward-center-config.json
{
  "mathematical_operand": "Multiple",
  "payout_numeral": 2,
  "seller_reward_payout_basis_points": 200
}
```

```
reward-center-cli create -c <CONFIG_FILE> -a <AUCTION_HOUSE> -M <MINT_REWARDS> -k <KEYPAIR> -r <RPC>
```

2. Fund the reward center treasury so it can distribute token to buyers and sellers. The keypair should be a wallet that has SPL token.

```
reward-center-cli fund -R <REWARD_CENTER> -a <AMOUNT> -k <KEYPAIR> -r <RPC>
```

3. The reward center plugin for the metaplex JS sdk makes use of an address lookup table for reducing the number accounts needed to be sent with each transaction. Create a address lookup table with the following command.

```
reward-center-cli create-alt --keypair <KEYPAIR> --rpc <RPC> --auction-house <AUCTION_HOUSE>
```