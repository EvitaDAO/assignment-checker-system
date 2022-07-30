use anchor_lang::prelude::*;
use anchor_lang::solana_program::pubkey::PUBKEY_BYTES;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{mint_to, Mint, MintTo, Token, TokenAccount};
pub use assignment_checker::{assignment_checker_canonical_pda, check_result_canonical_pda};
use assignment_checker::{
    cpi::accounts::Check, program::AssignmentChecker, AssignmentCheckerState,
};

use course_manager::Course;
use program::CourseBatchManager;

declare_id!("Po3YrSjzp5HM7VRFYszFM23LVJ58HHC9qoionaUgvRy");

pub const COURSE_DATA_SEED: &[u8; 11] = b"course_data";
pub const BATCH_DATA_SEED: &[u8; 10] = b"batch_data";
pub const BATCH_MINT_SEED: &[u8; 10] = b"batch_mint";
pub const BATCH_ID_SEED: &[u8; 15] = b"course_batch_id";
pub const ASSIGNMENT_ID_SEED: &[u8; 13] = b"assignment_id";
pub const STUDENT_ADDRESS_SEED: &[u8; 15] = b"student_address";

#[program]
pub mod course_batch_manager {
    use super::*;

    /// Create data and mint accounts for course batch
    pub fn create_new_batch(ctx: Context<NewCourseBatch>, batch_id: [u8; 16]) -> Result<()> {
        let course_batch_account = &mut ctx.accounts.course_batch;
        course_batch_account.id = batch_id;
        course_batch_account.course = ctx.accounts.course.key();
        course_batch_account.authority = ctx.accounts.authority.key();
        course_batch_account.mint = ctx.accounts.mint.key();
        course_batch_account.bump_seed = *ctx
            .bumps
            .get("course_batch")
            .expect("course_batch pda is present");
        Ok(())
    }

    /// Create Associated Token Account for given student wallet and mint
    pub fn enroll_batch(_ctx: Context<EnrollBatch>) -> Result<()> {
        // ATA is inited by Anchor
        Ok(())
    }

    pub fn check_assignment(
        ctx: Context<CheckAssignment>,
        assignment_id: [u8; 16],
        expected_hash_chain_length: u16,
        hash_chain_tail_parent: [u8; 32],
    ) -> Result<()> {
        let check = ctx.accounts;

        let course_key = check.course.key();

        let assignment_checker_seeds = [
            assignment_checker::COURSE_DATA_SEED,
            course_key.as_ref(),
            assignment_checker::ASSIGNMENT_ID_SEED,
            assignment_id.as_ref(),
            &[check.assignment_checker.bump_seed],
        ];

        let student_key = check.student.key();

        let check_result_seeds = [
            STUDENT_ADDRESS_SEED,
            student_key.as_ref(),
            COURSE_DATA_SEED,
            course_key.as_ref(),
            ASSIGNMENT_ID_SEED,
            assignment_id.as_ref(),
            &[*ctx
                .bumps
                .get("check_result")
                .expect("check_result pda is present")],
        ];

        let signer_seeds = [
            // assignment checker seeds
            assignment_checker_seeds.as_slice(),
            check_result_seeds.as_slice(),
        ];

        assignment_checker::cpi::check(
            check.check_cpi_ctx(signer_seeds.as_slice()),
            assignment_id,
            expected_hash_chain_length,
            hash_chain_tail_parent,
        )?;

        // let check_result = &check.check_result;
        // if check_result.check_passed && check_result.passed_first_time {
        //     let cpi_ctx = CpiContext::new(
        //         check.token_program.to_account_info(),
        //         MintTo {
        //             mint: check.mint.to_account_info(),
        //             to: check.student.to_account_info(),
        //             authority: check.course_batch_manager_program.to_account_info(),
        //         },
        //     );
        //     mint_to(
        //         cpi_ctx,
        //         check.assignment_checker.to_mint_on_successful_check.into(),
        //     )?;
        // }
        Ok(())
    }
}

pub fn batch_canonical_pda(course_address: Pubkey, batch_id: &[u8; 16]) -> Pubkey {
    Pubkey::find_program_address(
        &[
            COURSE_DATA_SEED,
            course_address.as_ref(),
            BATCH_ID_SEED,
            batch_id,
            BATCH_DATA_SEED,
        ],
        &ID,
    )
    .0
}

pub fn batch_mint_canonical_pda(course_address: Pubkey, batch_id: &[u8; 16]) -> Pubkey {
    Pubkey::find_program_address(
        &[
            COURSE_DATA_SEED,
            course_address.as_ref(),
            BATCH_ID_SEED,
            batch_id,
            BATCH_MINT_SEED,
        ],
        &ID,
    )
    .0
}

pub fn check_result_canonical_pda(
    student_address: Pubkey,
    course_data: Pubkey,
    assignment_id: &[u8; 16],
) -> Pubkey {
    Pubkey::find_program_address(
        &[
            STUDENT_ADDRESS_SEED,
            student_address.as_ref(),
            COURSE_DATA_SEED,
            course_data.as_ref(),
            ASSIGNMENT_ID_SEED,
            assignment_id,
        ],
        &ID,
    )
    .0
}

#[derive(Accounts)]
#[instruction(batch_id: [u8; 16])]
pub struct NewCourseBatch<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    pub course: Account<'info, Course>,
    #[account(init, payer = authority, space = 8 + CourseBatch::LEN, seeds= [
        COURSE_DATA_SEED,
        course.key().as_ref(),
        BATCH_ID_SEED,
        &batch_id,
        BATCH_DATA_SEED,
    ], bump)]
    pub course_batch: Account<'info, CourseBatch>,
    #[account(init, payer = authority,
        mint::authority = ID,
        mint::decimals = 0,
        mint::freeze_authority = ID,
        seeds= [
            COURSE_DATA_SEED,
            course.key().as_ref(),
            BATCH_ID_SEED,
            &batch_id,
            BATCH_MINT_SEED,
    ], bump)]
    pub mint: Account<'info, Mint>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct EnrollBatch<'info> {
    #[account(mut)]
    pub student: Signer<'info>,
    // course authority
    pub authority: AccountInfo<'info>,
    #[account(owner = ID, has_one = authority, has_one = mint)]
    pub course_batch: Account<'info, CourseBatch>,
    pub mint: Account<'info, Mint>,
    #[account(
        init,
        payer = student,
        associated_token::mint = mint,
        associated_token::authority = student,
    )]
    pub course_batch_token: Account<'info, TokenAccount>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction(batch_id: [u8; 16], assignment_id: [u8; 16])]
pub struct CheckAssignment<'info> {
    #[account(mut)]
    pub student: Signer<'info>,
    // course authority
    pub authority: AccountInfo<'info>,
    #[account(owner = course_manager::ID, has_one = authority)]
    pub course: Account<'info, Course>,
    // CHECK: pda will check it
    #[account(mut)]
    pub assignment_checker: Account<'info, AssignmentCheckerState>,
    #[account(init_if_needed, payer = student, space = 8 + AssignmentCheckerState::LEN,
        seeds=[
            STUDENT_ADDRESS_SEED,
            student.key().as_ref(),
            COURSE_DATA_SEED,
            course.key().as_ref(),
            assignment_checker::ASSIGNMENT_ID_SEED,
            assignment_id.as_ref(),
    ], bump)]
    pub check_result: Account<'info, assignment_checker::CheckResult>,
    #[account(
        mint::authority = ID,
        mint::decimals = 0,
        seeds= [
            COURSE_DATA_SEED,
            course.key().as_ref(),
            BATCH_ID_SEED,
            &batch_id,
            BATCH_MINT_SEED,
    ], bump)]
    pub mint: Account<'info, Mint>,
    #[account(
        associated_token::mint = mint,
        associated_token::authority = student,
    )]
    pub course_batch_token: Account<'info, TokenAccount>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub assignment_checker_program: Program<'info, AssignmentChecker>,
    pub course_batch_manager_program: Program<'info, CourseBatchManager>,
}

impl<'a, 'b, 'c, 'info> CheckAssignment<'info> {
    pub fn check_cpi_ctx(
        &self,
        signer_seeds: &'a [&'b [&'c [u8]]],
    ) -> CpiContext<'a, 'b, 'c, 'info, Check<'info>> {
        let cpi_program = self.assignment_checker_program.to_account_info();

        let course = self.course.to_account_info();
        let check_result = self.check_result.to_account_info();
        let cpi_accounts = Check {
            student: self.student.to_account_info(),
            course,
            assignment_checker: self.assignment_checker.to_account_info(),
            check_result,
            system_program: self.system_program.to_account_info(),
        };
        CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds)
    }
}
/// Each batch of any course has unique id
///
/// Course authority creates course batch account on each batch of course.
#[account]
pub struct CourseBatch {
    /// Course batch identifier like UUID or ULID
    pub id: [u8; 16],
    pub course: Pubkey,
    /// Course batch organizer
    pub authority: Pubkey,
    /// Mint account
    pub mint: Pubkey,
    pub bump_seed: u8,
}

impl CourseBatch {
    pub const LEN: usize = 16 + PUBKEY_BYTES * 3 + 1;
}
