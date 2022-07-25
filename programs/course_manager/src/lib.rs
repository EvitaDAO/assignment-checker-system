use anchor_lang::prelude::*;
use anchor_lang::solana_program::pubkey::PUBKEY_BYTES;

declare_id!("Po2hjSPEQmN9e1YLiZwwL3tCkqMCo2wYyLqAkF7ZmQn");

pub const COURSE_AUTHORITY_SEED: &[u8; 16] = b"course_authority";
pub const COURSE_ID_SEED: &[u8; 9] = b"course_id";

#[program]
pub mod course_manager {
    use super::*;

    pub fn create_new_course(ctx: Context<NewCourse>, course_id: [u8; 16]) -> Result<()> {
        let course_account = &mut ctx.accounts.course;
        course_account.id = course_id;
        course_account.authority = ctx.accounts.course_authority.key();
        course_account.bump_seed = *ctx.bumps.get("course").expect("course pda is present");
        Ok(())
    }
}

pub fn course_canonical_pda(course_authority: Pubkey, course_id: &[u8; 16]) -> Pubkey {
    Pubkey::find_program_address(
        &[
            COURSE_AUTHORITY_SEED,
            course_authority.as_ref(),
            COURSE_ID_SEED,
            course_id,
        ],
        &ID,
    )
    .0
}

#[derive(Accounts)]
#[instruction(course_id: [u8; 16])]
pub struct NewCourse<'info> {
    #[account(mut)]
    pub course_authority: Signer<'info>,
    // By default init sets the owner field of the created account to the currently executing program.
    // Anchor will find the canonical bump for the assignment checker PDA.
    // The PDA is derived from course_authority account and course name.
    // #[account(init, payer = course_authority, space = 8 + Course::LEN)]
    #[account(init, payer = course_authority, space = 8 + Course::LEN, seeds=[
        COURSE_AUTHORITY_SEED,
        course_authority.key().as_ref(),
        COURSE_ID_SEED,
        &course_id
    ], bump)]
    pub course: Account<'info, Course>,
    pub system_program: Program<'info, System>,
}

/// Each course has unique id
///
/// Course authority creates course account before the first batch of new course.
#[account]
pub struct Course {
    /// Course identifier like UUID or ULID
    pub id: [u8; 16],
    /// Course organizer
    pub authority: Pubkey,
    pub bump_seed: u8,
}

impl Course {
    pub const LEN: usize = 16 + PUBKEY_BYTES + 1;
}
