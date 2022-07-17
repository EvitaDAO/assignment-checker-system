use anchor_lang::prelude::*;

use anchor_spl::token::Mint;
use course_manager::program::CourseManager;

pub const COURSE_ACCOUNT_SEED: &[u8; 14] = b"course_account";
pub const BATCH_ID_SEED: &[u8; 8] = b"batch_id";
pub const ASSIGNMENT_ID_SEED: &[u8; 13] = b"assignment_id";

declare_id!("Po1RaS8BEDbNcn5oXsFryAeQ6Wn8fvmE111DJaKCgPC");
#[program]
pub mod assignment_checker {
    use super::*;

    pub fn create(
        ctx: Context<Create>,
        batch_id: u16,
        assignment_id: u16,
        to_mint_on_successful_check: u16,
        hash_chain_length: u16,
        salt: [u8; 32],
        ground_truth_hash_chain_tail: [u8; 32],
    ) -> Result<()> {
        let checker_account = &mut ctx.accounts.assignment_checker;
        checker_account.batch_id = batch_id;
        checker_account.assignment_id = assignment_id;
        checker_account.to_mint_on_successful_check = to_mint_on_successful_check;
        checker_account.hash_chain_length = hash_chain_length;
        checker_account.salt = salt;
        checker_account.ground_truth_hash_chain_tail = ground_truth_hash_chain_tail;
        checker_account.bump_seed = *ctx
            .bumps
            .get("assignment_checker")
            .expect("assignment_checker pda is present");
        Ok(())
    }
}

// validation struct
#[derive(Accounts)]
#[instruction(batch_id: u16, assignment_id: u16)]
pub struct Create<'info> {
    #[account(mut)]
    pub course_authority: Signer<'info>,
    pub course_program: Program<'info, CourseManager>,
    pub course_account: Account<'info, course_manager::Course>,
    // By default init sets the owner field of the created account to the currently executing program.
    // Anchor will find the canonical bump for the assignment checker PDA.
    // The PDA is derived from course account, batch and assignment IDs.
    #[account(init, payer = course_authority, space = 8 + AssignmentChecker::LEN, seeds=[
        COURSE_ACCOUNT_SEED,
        course_account.key().as_ref(),
        BATCH_ID_SEED,
        batch_id.to_be_bytes().as_ref(),
        ASSIGNMENT_ID_SEED,
        assignment_id.to_be_bytes().as_ref(),
    ], bump)]
    pub assignment_checker: Account<'info, AssignmentChecker>,
    // #[account(
    //     mint::authority = course_authority,
    // )]
    // pub mint_account: Account<'info, Mint>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct AssignmentChecker {
    /// Batch identifies the given course run
    batch_id: u16,
    /// Assignment ID is unique within a course batch
    assignment_id: u16,
    to_mint_on_successful_check: u16,
    hash_chain_length: u16,
    salt: [u8; 32],
    ground_truth_hash_chain_tail: [u8; 32],
    bump_seed: u8,
}

impl AssignmentChecker {
    pub const LEN: usize = 2 + 2 + 2 + 2 + 32 + 32 + 1;
}
