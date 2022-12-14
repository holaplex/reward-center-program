/**
 * This code was GENERATED using the solita package.
 * Please DO NOT EDIT THIS FILE, instead rerun solita to update it or write a wrapper to add functionality.
 *
 * See: https://github.com/metaplex-foundation/solita
 */

import * as beet from '@metaplex-foundation/beet';
export type CloseOfferParams = {
  escrowPaymentBump: number;
};

/**
 * @category userTypes
 * @category generated
 */
export const closeOfferParamsBeet = new beet.BeetArgsStruct<CloseOfferParams>(
  [['escrowPaymentBump', beet.u8]],
  'CloseOfferParams',
);
