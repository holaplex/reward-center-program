/**
 * This code was GENERATED using the solita package.
 * Please DO NOT EDIT THIS FILE, instead rerun solita to update it or write a wrapper to add functionality.
 *
 * See: https://github.com/metaplex-foundation/solita
 */

import * as beet from '@metaplex-foundation/beet';
import { PayoutOperation, payoutOperationBeet } from './PayoutOperation';
export type RewardRules = {
  sellerRewardPayoutBasisPoints: number;
  mathematicalOperand: PayoutOperation;
  payoutNumeral: number;
};

/**
 * @category userTypes
 * @category generated
 */
export const rewardRulesBeet = new beet.BeetArgsStruct<RewardRules>(
  [
    ['sellerRewardPayoutBasisPoints', beet.u16],
    ['mathematicalOperand', payoutOperationBeet],
    ['payoutNumeral', beet.u16],
  ],
  'RewardRules',
);
