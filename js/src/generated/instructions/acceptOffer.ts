/**
 * This code was GENERATED using the solita package.
 * Please DO NOT EDIT THIS FILE, instead rerun solita to update it or write a wrapper to add functionality.
 *
 * See: https://github.com/metaplex-foundation/solita
 */

import * as splToken from '@solana/spl-token';
import * as beet from '@metaplex-foundation/beet';
import * as web3 from '@solana/web3.js';
import { AcceptOfferParams, acceptOfferParamsBeet } from '../types/AcceptOfferParams';

/**
 * @category Instructions
 * @category AcceptOffer
 * @category generated
 */
export type AcceptOfferInstructionArgs = {
  acceptOfferParams: AcceptOfferParams;
};
/**
 * @category Instructions
 * @category AcceptOffer
 * @category generated
 */
export const acceptOfferStruct = new beet.BeetArgsStruct<
  AcceptOfferInstructionArgs & {
    instructionDiscriminator: number[] /* size: 8 */;
  }
>(
  [
    ['instructionDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)],
    ['acceptOfferParams', acceptOfferParamsBeet],
  ],
  'AcceptOfferInstructionArgs',
);
/**
 * Accounts required by the _acceptOffer_ instruction
 *
 * @property [_writable_] buyer
 * @property [_writable_] buyerRewardTokenAccount
 * @property [_writable_] seller
 * @property [_writable_] sellerRewardTokenAccount
 * @property [_writable_] offer
 * @property [_writable_] tokenAccount
 * @property [] tokenMint
 * @property [] metadata
 * @property [] treasuryMint
 * @property [_writable_] sellerPaymentReceiptAccount
 * @property [_writable_] buyerReceiptTokenAccount
 * @property [] authority
 * @property [_writable_] escrowPaymentAccount
 * @property [] auctionHouse
 * @property [_writable_] auctionHouseFeeAccount
 * @property [_writable_] auctionHouseTreasury
 * @property [_writable_] buyerTradeState
 * @property [_writable_] sellerTradeState
 * @property [_writable_] freeSellerTradeState
 * @property [] rewardCenter
 * @property [_writable_] rewardCenterRewardTokenAccount
 * @property [] ahAuctioneerPda
 * @property [] programAsSigner
 * @property [] auctionHouseProgram
 * @category Instructions
 * @category AcceptOffer
 * @category generated
 */
export type AcceptOfferInstructionAccounts = {
  buyer: web3.PublicKey;
  buyerRewardTokenAccount: web3.PublicKey;
  seller: web3.PublicKey;
  sellerRewardTokenAccount: web3.PublicKey;
  offer: web3.PublicKey;
  tokenAccount: web3.PublicKey;
  tokenMint: web3.PublicKey;
  metadata: web3.PublicKey;
  treasuryMint: web3.PublicKey;
  sellerPaymentReceiptAccount: web3.PublicKey;
  buyerReceiptTokenAccount: web3.PublicKey;
  authority: web3.PublicKey;
  escrowPaymentAccount: web3.PublicKey;
  auctionHouse: web3.PublicKey;
  auctionHouseFeeAccount: web3.PublicKey;
  auctionHouseTreasury: web3.PublicKey;
  buyerTradeState: web3.PublicKey;
  sellerTradeState: web3.PublicKey;
  freeSellerTradeState: web3.PublicKey;
  rewardCenter: web3.PublicKey;
  rewardCenterRewardTokenAccount: web3.PublicKey;
  ahAuctioneerPda: web3.PublicKey;
  programAsSigner: web3.PublicKey;
  auctionHouseProgram: web3.PublicKey;
  tokenProgram?: web3.PublicKey;
  systemProgram?: web3.PublicKey;
  ataProgram?: web3.PublicKey;
  rent?: web3.PublicKey;
  anchorRemainingAccounts?: web3.AccountMeta[];
};

export const acceptOfferInstructionDiscriminator = [227, 82, 234, 131, 1, 18, 48, 2];

/**
 * Creates a _AcceptOffer_ instruction.
 *
 * @param accounts that will be accessed while the instruction is processed
 * @param args to provide as instruction data to the program
 *
 * @category Instructions
 * @category AcceptOffer
 * @category generated
 */
export function createAcceptOfferInstruction(
  accounts: AcceptOfferInstructionAccounts,
  args: AcceptOfferInstructionArgs,
  programId = new web3.PublicKey('RwDDvPp7ta9qqUwxbBfShsNreBaSsKvFcHzMxfBC3Ki'),
) {
  const [data] = acceptOfferStruct.serialize({
    instructionDiscriminator: acceptOfferInstructionDiscriminator,
    ...args,
  });
  const keys: web3.AccountMeta[] = [
    {
      pubkey: accounts.buyer,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: accounts.buyerRewardTokenAccount,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: accounts.seller,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: accounts.sellerRewardTokenAccount,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: accounts.offer,
      isWritable: true,
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
      pubkey: accounts.metadata,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: accounts.treasuryMint,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: accounts.sellerPaymentReceiptAccount,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: accounts.buyerReceiptTokenAccount,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: accounts.authority,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: accounts.escrowPaymentAccount,
      isWritable: true,
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
      pubkey: accounts.auctionHouseTreasury,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: accounts.buyerTradeState,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: accounts.sellerTradeState,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: accounts.freeSellerTradeState,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: accounts.rewardCenter,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: accounts.rewardCenterRewardTokenAccount,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: accounts.ahAuctioneerPda,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: accounts.programAsSigner,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: accounts.auctionHouseProgram,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: accounts.tokenProgram ?? splToken.TOKEN_PROGRAM_ID,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: accounts.systemProgram ?? web3.SystemProgram.programId,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: accounts.ataProgram ?? splToken.ASSOCIATED_TOKEN_PROGRAM_ID,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: accounts.rent ?? web3.SYSVAR_RENT_PUBKEY,
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