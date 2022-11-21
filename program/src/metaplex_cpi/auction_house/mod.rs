use anchor_lang::prelude::*;
use mpl_auction_house::id;
use solana_program::instruction::Instruction;

pub struct AuctioneerInstructionArgs<'c, 'info, T> {
    pub accounts: T,
    pub instruction_data: Vec<u8>,
    pub auctioneer_authority: Pubkey,
    pub remaining_accounts: Option<&'c [AccountInfo<'info>]>,
}

pub fn make_auctioneer_instruction<'c, 'info, T: ToAccountInfos<'info> + ToAccountMetas>(
    AuctioneerInstructionArgs {
        accounts,
        instruction_data,
        auctioneer_authority,
        remaining_accounts,
    }: AuctioneerInstructionArgs<'c, 'info, T>,
) -> (Instruction, Vec<AccountInfo<'info>>) {
    let mut account_infos = accounts.to_account_infos();

    let mut accounts: Vec<AccountMeta> = accounts
        .to_account_metas(None)
        .into_iter()
        .zip(account_infos.clone())
        .map(|mut pair| {
            pair.0.is_signer = pair.1.is_signer;
            if pair.0.pubkey == auctioneer_authority {
                pair.0.is_signer = true;
            }
            pair.0
        })
        .collect();

    if let Some(remaining_accounts) = remaining_accounts {
        accounts.append(&mut remaining_accounts.to_vec().to_account_metas(None));
        account_infos.append(&mut remaining_accounts.to_vec());
    };

    (
        Instruction {
            program_id: id(),
            data: instruction_data,
            accounts,
        },
        account_infos,
    )
}
