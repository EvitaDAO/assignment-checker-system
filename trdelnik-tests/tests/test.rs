use fehler::throws;
use program_client::assignment_checker_instruction;
use program_client::course_manager_instruction;
use trdelnik_client::{anyhow::Result, *};
// @todo: do not forget to import your program crate (also in the ../Cargo.toml)

// @todo: create and deploy your fixture
#[throws]
#[fixture]
async fn init_fixture() -> Fixture {
    let mut f = Fixture::new();
    // Deploy
    f.deploy().await?;
    f.client
        .airdrop(f.course_authority.pubkey(), 5_000_000)
        .await?;

    let (course_address, _) = Pubkey::find_program_address(
        &[
            course_manager::COURSE_AUTHORITY_SEED,
            f.course_authority.pubkey().as_ref(),
            course_manager::COURSE_ID_SEED,
            &f.course_id,
        ],
        &f.course_program.pubkey(),
    );
    f.course_pda = course_address;
    // create course account
    course_manager_instruction::create_new_course(
        &f.client,
        f.course_id,
        f.course_authority.pubkey(),
        f.course_pda.clone(),
        f.system_program,
        [f.course_authority.clone()],
    )
    .await?;

    // // Assignment checker can mint course tokens on successful assignment checks
    f.client
        .create_token_mint(&f.mint_keypair, f.course_authority.pubkey(), None, 0)
        .await?;
    // These token accounts could be created by the course program when a student has enrolled to the course.
    // Student_a account for the course token.
    f.student_a_token_account = f
        .client
        .create_associated_token_account(&f.student_a, f.mint_keypair.pubkey())
        .await?;
    // Student_b account for the course token.
    f.student_b_token_account = f
        .client
        .create_associated_token_account(&f.student_b, f.mint_keypair.pubkey())
        .await?;
    f
}

#[trdelnik_test]
async fn test_happy_path(#[future] init_fixture: Result<Fixture>) {
    // @todo: add your happy path test scenario and the other test cases
    let f = init_fixture.await?;
    let course_pda = f.get_course_account().await?;
    assert_eq!(course_pda.authority, f.course_authority.pubkey());

    let batch_id: u16 = 1;
    let assignment_id: u16 = 1;
    let (assignment_checker, _) = Pubkey::find_program_address(
        &[
            assignment_checker::COURSE_ACCOUNT_SEED,
            f.course_pda.as_ref(),
            assignment_checker::BATCH_ID_SEED,
            batch_id.to_be_bytes().as_ref(),
            assignment_checker::ASSIGNMENT_ID_SEED,
            assignment_id.to_be_bytes().as_ref(),
        ],
        &f.program.pubkey(),
    );
    assignment_checker_instruction::create(
        &f.client,
        1,
        1,
        10,
        100,
        [0; 32],
        [1; 32],
        f.course_authority.pubkey().clone(),
        f.course_program.pubkey(),
        f.course_pda,
        assignment_checker,
        // f.mint_keypair.pubkey(),
        f.system_program,
        [f.course_authority.clone()],
    )
    .await?;
}

// @todo: design and implement all the logic you need for your fixture(s)
struct Fixture {
    client: Client,
    program: Keypair,
    system_program: Pubkey,
    course_program: Keypair,
    course_authority: Keypair,
    course_id: [u8; 16],
    course_pda: Pubkey,
    // can be related to course_authority
    mint_keypair: Keypair,
    student_a: Keypair,
    student_a_token_account: Pubkey,
    student_b: Keypair,
    student_b_token_account: Pubkey,
}
impl Fixture {
    fn new() -> Self {
        Fixture {
            client: Client::new(system_keypair(0)),
            program: program_keypair(1),
            system_program: anchor_lang::system_program::ID,
            course_program: program_keypair(2),
            course_authority: keypair(0),
            course_id: *b"web2_to_web3____",
            course_pda: Default::default(),
            mint_keypair: keypair(2),
            student_a: keypair(3),
            student_a_token_account: Pubkey::default(),
            student_b: keypair(4),
            student_b_token_account: Pubkey::default(),
        }
    }

    #[throws]
    async fn deploy(&mut self) {
        self.client
            .deploy_by_name(&self.course_program, "course_manager")
            .await?;
        self.client
            .deploy_by_name(&self.program, "assignment_checker")
            .await?;
    }

    #[throws]
    async fn get_course_account(&self) -> course_manager::Course {
        self.client
            .account_data::<course_manager::Course>(self.course_pda)
            .await?
    }
}
