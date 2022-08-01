use anchor_lang::prelude::*;
use anchor_lang::solana_program::pubkey::PUBKEY_BYTES;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{mint_to, Mint, MintTo, Token, TokenAccount};
use assignment_checker::{
    cpi::accounts::{Check, Init, InitCheckResult},
    program::AssignmentChecker,
};
pub use assignment_checker::{AssignmentCheckerState, CheckResult};

use course_manager::Course;

declare_id!("Po3YrSjzp5HM7VRFYszFM23LVJ58HHC9qoionaUgvRy");

pub const COURSE_DATA_SEED: &[u8; 11] = assignment_checker::COURSE_DATA_SEED;
pub const BATCH_DATA_SEED: &[u8; 10] = b"batch_data";
pub const BATCH_MINT_SEED: &[u8; 10] = b"batch_mint";
pub const BATCH_ID_SEED: &[u8; 15] = b"course_batch_id";
pub const ASSIGNMENT_ID_SEED: &[u8; 13] = assignment_checker::ASSIGNMENT_ID_SEED;
pub const STUDENT_ADDRESS_SEED: &[u8; 15] = assignment_checker::STUDENT_ADDRESS_SEED;

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
        course_batch_account.mint_bump_seed = *ctx.bumps.get("mint").expect("mint pda is present");
        Ok(())
    }

    /// Create Associated Token Account for given student wallet and mint
    pub fn enroll_batch(_ctx: Context<EnrollBatch>) -> Result<()> {
        // ATA is inited by Anchor
        Ok(())
    }

    /// Create an assignment checker
    pub fn create_assignment_checker(
        ctx: Context<CreateAssignmentChecker>,
        assignment_id: [u8; 16],
        hash_chain_length: u16,
        to_mint_on_successful_check: u16,
        salt: [u8; 32],
        // Creator of assignment checker is a trusted authority
        // It should precompute ground truth hash chain tail
        // to save nonfree compute operations of onchain program
        // and not to send the ground truth assignment result value to public blockchain
        ground_truth_hash_chain_tail: [u8; 32],
    ) -> Result<()> {
        // we don't own assignment_checker account
        let create = ctx.accounts;

        let course_key = create.course.key();
        let assignment_checker_seeds = [
            COURSE_DATA_SEED,
            course_key.as_ref(),
            ASSIGNMENT_ID_SEED,
            assignment_id.as_ref(),
            &[*ctx
                .bumps
                .get("assignment_checker")
                .expect("assignment_checker pda is present")],
        ];

        let signer_seeds = [assignment_checker_seeds.as_slice()];

        assignment_checker::cpi::init(
            create.init_cpi_ctx(signer_seeds.as_slice()),
            assignment_id,
            hash_chain_length,
            to_mint_on_successful_check,
            salt,
            ground_truth_hash_chain_tail,
        )?;
        Ok(())
    }

    /// Start assignment solving
    ///
    /// CheckResult account is initialized
    ///
    /// Called by a student when he/she starts to solve the assignment
    pub fn create_check_result(
        ctx: Context<CreateCheckResult>,
        assignment_id: [u8; 16],
    ) -> Result<()> {
        let create = ctx.accounts;

        let student_key = create.student.key();
        let course_key = create.course.key();
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

        let signer_seeds = [check_result_seeds.as_slice()];

        assignment_checker::cpi::init_check_result(
            create.init_check_result_cpi_ctx(signer_seeds.as_slice()),
            assignment_id,
        )?;
        Ok(())
    }

    /// Check assignment solution and mint `assignment_checker.to_mint_on_successful_check` tokens when the check is succeded
    pub fn check_assignment(
        ctx: Context<CheckAssignment>,
        expected_hash_chain_length: u16,
        hash_chain_tail_parent: [u8; 32],
    ) -> Result<()> {
        let check = ctx.accounts;

        let course_key = check.course.key();
        let assignment_checker_seeds = [
            COURSE_DATA_SEED,
            course_key.as_ref(),
            ASSIGNMENT_ID_SEED,
            check.assignment_checker.assignment_id.as_ref(),
            &[check.assignment_checker.bump_seed],
        ];

        let student_key = check.student.key();

        let check_result_seeds = [
            assignment_checker::STUDENT_ADDRESS_SEED,
            student_key.as_ref(),
            assignment_checker::COURSE_DATA_SEED,
            course_key.as_ref(),
            assignment_checker::ASSIGNMENT_ID_SEED,
            check.check_result.assignment_id.as_ref(),
            &[check.check_result.bump_seed],
        ];
        let signer_seeds = [
            assignment_checker_seeds.as_slice(),
            check_result_seeds.as_slice(),
        ];

        assignment_checker::cpi::check(
            check.check_cpi_ctx(signer_seeds.as_slice()),
            expected_hash_chain_length,
            hash_chain_tail_parent,
        )?;

        // deserialize check_result again after assignment checker has changed the account
        check.check_result.reload()?;

        let check_result = &check.check_result;
        msg!(
            "check_passed: {}, passed_first_time: {}",
            check_result.check_passed,
            check_result.passed_first_time
        );
        if check_result.check_passed && check_result.passed_first_time {
            let mint_seeds = [
                COURSE_DATA_SEED,
                course_key.as_ref(),
                BATCH_ID_SEED,
                check.course_batch.id.as_ref(),
                BATCH_MINT_SEED,
                &[check.course_batch.mint_bump_seed],
            ];
            let course_batch_seeds = [
                COURSE_DATA_SEED,
                course_key.as_ref(),
                BATCH_ID_SEED,
                check.course_batch.id.as_ref(),
                BATCH_DATA_SEED,
                &[check.course_batch.bump_seed],
            ];
            let signer_seeds = [mint_seeds.as_slice(), course_batch_seeds.as_slice()];
            let amount = check.assignment_checker.to_mint_on_successful_check.into();
            mint_to(check.mint_to_cpi_ctx(signer_seeds.as_slice()), amount)?;
            msg!("minted {} tokens to {}", amount, check.student.key());
        }
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

pub fn assignment_checker_canonical_pda(
    course_address: Pubkey,
    assignment_id: &[u8; 16],
) -> Pubkey {
    Pubkey::find_program_address(
        &[
            COURSE_DATA_SEED,
            course_address.as_ref(),
            ASSIGNMENT_ID_SEED,
            assignment_id,
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
    #[account(has_one = authority)]
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
        mint::authority = course_batch,
        mint::decimals = 0,
        mint::freeze_authority = course_batch,
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
    #[account(has_one = authority, has_one = mint)]
    pub course_batch: Account<'info, CourseBatch>,
    #[account(mint::authority = course_batch)]
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
#[instruction(assignment_id: [u8; 16], hash_chain_length: u16)]
pub struct CreateAssignmentChecker<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(has_one = authority)]
    pub course: Account<'info, course_manager::Course>,
    // By default init sets the owner field of the created account to the currently executing program.
    // We override it to assignment_checker program.
    // Anchor will find the canonical bump for the assignment checker PDA derived for course_batch_manager program.
    // Course batch manager can sign for PDA to do mutable cross-program operations.
    // The PDA is derived from course account and assignment IDs.
    #[account(init, owner = assignment_checker_program.key(), payer = authority, space = 8 + AssignmentCheckerState::LEN, seeds=[
        COURSE_DATA_SEED,
        course.key().as_ref(),
        ASSIGNMENT_ID_SEED,
        assignment_id.as_ref(),
    ], bump, constraint = hash_chain_length >= 2)]
    pub assignment_checker: Account<'info, AssignmentCheckerState>,
    pub assignment_checker_program: Program<'info, AssignmentChecker>,
    pub course_batch_manager_program: Program<'info, program::CourseBatchManager>,
    pub system_program: Program<'info, System>,
}

impl<'a, 'b, 'c, 'info> CreateAssignmentChecker<'info> {
    pub fn init_cpi_ctx(
        &self,
        signer_seeds: &'a [&'b [&'c [u8]]],
    ) -> CpiContext<'a, 'b, 'c, 'info, Init<'info>> {
        let cpi_program = self.assignment_checker_program.to_account_info();

        let cpi_accounts = Init {
            authority: self.authority.to_account_info(),
            course: self.course.to_account_info(),
            assignment_checker: self.assignment_checker.to_account_info(),
            result_processor_program: self.course_batch_manager_program.to_account_info(),
        };
        CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds)
    }
}

#[derive(Accounts)]
#[instruction(assignment_id: [u8; 16])]
pub struct CreateCheckResult<'info> {
    #[account(mut)]
    pub student: Signer<'info>,
    pub course: Account<'info, course_manager::Course>,

    #[account(init, payer = student, space = 8 + assignment_checker::CheckResult::LEN,
        owner = assignment_checker::ID,
        seeds=[
        STUDENT_ADDRESS_SEED,
        student.key().as_ref(),
        COURSE_DATA_SEED,
        course.key().as_ref(),
        ASSIGNMENT_ID_SEED,
        assignment_id.as_ref(),
    ], bump)]
    pub check_result: Account<'info, assignment_checker::CheckResult>,
    pub assignment_checker_program: Program<'info, AssignmentChecker>,
    pub course_batch_manager_program: Program<'info, program::CourseBatchManager>,
    pub system_program: Program<'info, System>,
}

impl<'a, 'b, 'c, 'info> CreateCheckResult<'info> {
    pub fn init_check_result_cpi_ctx(
        &self,
        signer_seeds: &'a [&'b [&'c [u8]]],
    ) -> CpiContext<'a, 'b, 'c, 'info, InitCheckResult<'info>> {
        let cpi_program = self.assignment_checker_program.to_account_info();

        let cpi_accounts = InitCheckResult {
            student: self.student.to_account_info(),
            course: self.course.to_account_info(),
            check_result: self.check_result.to_account_info(),
            result_processor_program: self.course_batch_manager_program.to_account_info(),
        };
        CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds)
    }
}
#[derive(Accounts)]
pub struct CheckAssignment<'info> {
    #[account(mut)]
    pub student: Signer<'info>,
    pub course: Account<'info, Course>,
    #[account(has_one = mint, constraint = course.authority == course_batch.authority,
        seeds=[
        COURSE_DATA_SEED,
        course.key().as_ref(),
        BATCH_ID_SEED,
        &course_batch.id,
        BATCH_DATA_SEED,
    ], bump=course_batch.bump_seed)]
    pub course_batch: Account<'info, CourseBatch>,
    // CHECK: pda check and assignment_id equality will be made by assignment_checker
    #[account(mut)]
    pub assignment_checker: Account<'info, AssignmentCheckerState>,

    // CHECK: pda check and assignment_id equality will be made by assignment_checker
    #[account(mut)]
    pub check_result: Account<'info, CheckResult>,
    #[account(
        mut,
        mint::authority = course_batch,
        mint::decimals = 0,
        seeds= [
            COURSE_DATA_SEED,
            course.key().as_ref(),
            BATCH_ID_SEED,
            &course_batch.id,
            BATCH_MINT_SEED,
    ], bump=course_batch.mint_bump_seed)]
    pub mint: Account<'info, Mint>,
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = student,
    )]
    pub course_batch_token: Account<'info, TokenAccount>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub assignment_checker_program: Program<'info, AssignmentChecker>,
    pub course_batch_manager_program: Program<'info, program::CourseBatchManager>,
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
            result_processor_program: self.course_batch_manager_program.to_account_info(),
        };
        CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds)
    }

    pub fn mint_to_cpi_ctx(
        &self,
        signer_seeds: &'a [&'b [&'c [u8]]],
    ) -> CpiContext<'a, 'b, 'c, 'info, MintTo<'info>> {
        let cpi_program = self.token_program.to_account_info();

        let cpi_accounts = MintTo {
            mint: self.mint.to_account_info(),
            to: self.course_batch_token.to_account_info(),
            authority: self.course_batch.to_account_info(),
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
    pub mint_bump_seed: u8,
}

impl CourseBatch {
    pub const LEN: usize = 16 + PUBKEY_BYTES * 3 + 1 + 1;
}
