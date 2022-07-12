use anchor_spl::token;
use assignmentchecker;
use fehler::throws;
use program_client::assignmentchecker_instruction;
use trdelnik_client::{anchor_lang::Key, anyhow::Result, *};
// @todo: do not forget to import your program crate (also in the ../Cargo.toml)

// @todo: create and deploy your fixture
#[throws]
#[fixture]
async fn init_fixture() -> Fixture {
    let mut f = Fixture::new();
    // Deploy
    f.deploy().await?;
    f.client
        .create_account_rent_exempt(&f.course_authority, 64, &f.system_program.pubkey())
        .await?;
    // Create program derived course_account. It could be managed by some external course program.
    f.course_account =
        Pubkey::find_program_address(&[b"course", b"web2_to_web3"], &f.course_program.pubkey()).0;
    // Assignment checker can mint course tokens on successful assignment checks
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
    assignmentchecker_instruction::create(
        &f.client,
        1,
        1,
        10,
        100,
        [0; 32],
        [1; 32],
        f.course_authority.pubkey().clone(),
        // f.course_account,
        // Pubkey::default(),
        // f.mint_keypair.pubkey(),
        // f.system_program.pubkey(),
        [f.course_authority.clone()],
    )
    .await?;
}

// @todo: design and implement all the logic you need for your fixture(s)
struct Fixture {
    client: Client,
    program: Keypair,
    system_program: Keypair,
    course_program: Keypair,
    course_authority: Keypair,
    course_account: Pubkey,
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
            system_program: system_keypair(1),
            course_program: program_keypair(1),
            course_authority: keypair(0),
            course_account: Pubkey::default(),
            mint_keypair: keypair(1),
            student_a: keypair(2),
            student_a_token_account: Pubkey::default(),
            student_b: keypair(3),
            student_b_token_account: Pubkey::default(),
        }
    }

    #[throws]
    async fn deploy(&mut self) {
        self.deploy_by_name(&self.program.clone(), "assignmentchecker")
            .await?;
    }
    #[throws]
    pub async fn deploy_by_name(&self, program_keypair: &Keypair, program_name: &str) {
        let reader = Reader::new();
        let mut program_data = reader
            .program_data(program_name)
            .await
            .expect("reading program data failed");

        // TODO: This will fail on devnet where airdrops are limited to 1 SOL
        self.client
            .airdrop(self.client.payer().pubkey(), 5_000_000_000)
            .await
            .expect("airdropping for deployment failed");

        self.client
            .deploy(program_keypair.clone(), std::mem::take(&mut program_data))
            .await
            .expect("deploying program failed");
    }
}
