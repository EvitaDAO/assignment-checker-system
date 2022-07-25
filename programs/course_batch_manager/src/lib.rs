use anchor_lang::prelude::*;
use anchor_lang::solana_program::pubkey::PUBKEY_BYTES;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{Mint, Token, TokenAccount};

use course_manager::Course;

declare_id!("Po3YrSjzp5HM7VRFYszFM23LVJ58HHC9qoionaUgvRy");

pub const COURSE_DATA_SEED: &[u8; 11] = b"course_data";
pub const BATCH_DATA_SEED: &[u8; 10] = b"batch_data";
pub const BATCH_MINT_SEED: &[u8; 10] = b"batch_mint";
pub const BATCH_ID_SEED: &[u8; 15] = b"course_batch_id";

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
