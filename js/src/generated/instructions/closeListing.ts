/**
 * This code was GENERATED using the solita package.
 * Please DO NOT EDIT THIS FILE, instead rerun solita to update it or write a wrapper to add functionality.
 *
 * See: https://github.com/metaplex-foundation/solita
 */

import * as splToken from '@solana/spl-token';
import * as beet from '@metaplex-foundation/beet';
import * as web3 from '@solana/web3.js';

/**
 * @category Instructions
 * @category CloseListing
 * @category generated
 */
export const closeListingStruct = new beet.BeetArgsStruct<{
  instructionDiscriminator: number[] /* size: 8 */;
}>(
  [['instructionDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)]],
  'CloseListingInstructionArgs',
);
/**
 * Accounts required by the _closeListing_ instruction
 *
 * @property [_writable_, **signer**] wallet
 * @property [_writable_] listing
 * @property [] metadata
 * @property [_writable_] tokenAccount
 * @property [] tokenMint
 * @property [] authority
 * @property [] rewardCenter
 * @property [] auctionHouse
 * @property [_writable_] auctionHouseFeeAccount
 * @property [_writable_] tradeState
 * @property [] ahAuctioneerPda
 * @property [] auctionHouseProgram
 * @category Instructions
 * @category CloseListing
 * @category generated
 */
export type CloseListingInstructionAccounts = {
  wallet: web3.PublicKey;
  listing: web3.PublicKey;
  metadata: web3.PublicKey;
  tokenAccount: web3.PublicKey;
  tokenMint: web3.PublicKey;
  authority: web3.PublicKey;
  rewardCenter: web3.PublicKey;
  auctionHouse: web3.PublicKey;
  auctionHouseFeeAccount: web3.PublicKey;
  tradeState: web3.PublicKey;
  ahAuctioneerPda: web3.PublicKey;
  tokenProgram?: web3.PublicKey;
  auctionHouseProgram: web3.PublicKey;
  anchorRemainingAccounts?: web3.AccountMeta[];
};

export const closeListingInstructionDiscriminator = [33, 15, 192, 81, 78, 175, 159, 97];

/**
 * Creates a _CloseListing_ instruction.
 *
 * @param accounts that will be accessed while the instruction is processed
 * @category Instructions
 * @category CloseListing
 * @category generated
 */
export function createCloseListingInstruction(
  accounts: CloseListingInstructionAccounts,
  programId = new web3.PublicKey('RwDDvPp7ta9qqUwxbBfShsNreBaSsKvFcHzMxfBC3Ki'),
) {
  const [data] = closeListingStruct.serialize({
    instructionDiscriminator: closeListingInstructionDiscriminator,
  });
  const keys: web3.AccountMeta[] = [
    {
      pubkey: accounts.wallet,
      isWritable: true,
      isSigner: true,
    },
    {
      pubkey: accounts.listing,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: accounts.metadata,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: accounts.tokenAccount,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: accounts.tokenMint,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: accounts.authority,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: accounts.rewardCenter,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: accounts.auctionHouse,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: accounts.auctionHouseFeeAccount,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: accounts.tradeState,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: accounts.ahAuctioneerPda,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: accounts.tokenProgram ?? splToken.TOKEN_PROGRAM_ID,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: accounts.auctionHouseProgram,
      isWritable: false,
      isSigner: false,
    },
  ];

  if (accounts.anchorRemainingAccounts != null) {
    for (const acc of accounts.anchorRemainingAccounts) {
      keys.push(acc);
    }
  }

  const ix = new web3.TransactionInstruction({
    programId,
    keys,
    data,
  });
  return ix;
}
