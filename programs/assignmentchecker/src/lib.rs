use anchor_lang::prelude::*;
use anchor_spl::token::Mint;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod assignmentchecker {
    use super::*;

    pub fn create(
        ctx: Context<Create>,
        to_mint_on_successful_check: u16,
        hash_chain_length: u16,
        salt: [u8; 32],
        ground_truth_hash_chain_tail: [u8; 32],
    ) -> Result<()> {
        let checker_account = &mut ctx.accounts.assignment_checker;
        checker_account.to_mint_on_successful_check = to_mint_on_successful_check;
        checker_account.hash_chain_length = hash_chain_length;
        checker_account.salt = salt;
        checker_account.ground_truth_hash_chain_tail = ground_truth_hash_chain_tail;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Create<'info> {
    // By default init sets the owner field of the created account to the currently executing program.
    #[account(init, payer = authority, space = 8 + 2 + 2 + 32 + 32)]
    pub assignment_checker: Account<'info, AssignmentCheck>,

    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        mint::authority = authority,
    )]
    pub mint_account: Account<'info, Mint>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct AssignmentCheck {
    to_mint_on_successful_check: u16,
    hash_chain_length: u16,
    salt: [u8; 32],
    ground_truth_hash_chain_tail: [u8; 32],
}
