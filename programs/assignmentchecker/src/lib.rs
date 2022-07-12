use anchor_lang::prelude::*;

use anchor_spl::token::Mint;

declare_id!("6syjPkpJf6gT6JCaXvWVDfKk4vv29s16F7FTWB2t4XP5");
#[program]
pub mod assignmentchecker {
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
        todo!();
        // let checker_account = &mut ctx.accounts.assignment_checker;
        // checker_account.batch_id = batch_id;
        // checker_account.assignment_id = assignment_id;
        // checker_account.to_mint_on_successful_check = to_mint_on_successful_check;
        // checker_account.hash_chain_length = hash_chain_length;
        // checker_account.salt = salt;
        // checker_account.ground_truth_hash_chain_tail = ground_truth_hash_chain_tail;
        Ok(())
    }
}

// validation struct
#[derive(Accounts)]
// #[instruction(checker: AssignmentChecker)]
pub struct Create<'info> {
    #[account(mut)]
    pub course_authority: Signer<'info>,
    // pub course_account: Account<'info, Course>,
    // By default init sets the owner field of the created account to the currently executing program.
    // Anchor will find the canonical bump for the assignment checker PDA.
    // The PDA is derived from course account, batch and assignment IDs.
    // #[account(init, payer = course_authority, space = 8 + 2 + 2 + 32 + 32, seeds=[
    //     b"assignment_checker".as_ref(),
    //     course_authority.key().as_ref(),
    //     b"course_account".as_ref(),
    //     course_account.key().as_ref(),
    //     b"batch_id".as_ref(),
    //     checker.batch_id.to_be_bytes().as_ref(),
    //     b"assignment_id".as_ref(),
    //     checker.assignment_id.to_be_bytes().as_ref(),
    // ], bump)]
    // pub assignment_checker: Account<'info, AssignmentChecker>,
    // #[account(
    //     mint::authority = course_authority,
    // )]
    // pub mint_account: Account<'info, Mint>,
    // pub system_program: Program<'info, System>,
}

/// Each batch of any course has unique account id
///
/// Course authority creates course account on the first batch of new course.
// #[account]
// pub struct Course {}

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
}
