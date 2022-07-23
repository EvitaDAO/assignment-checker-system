use anchor_lang::solana_program::blake3;
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
    f.client.airdrop(f.student_a.pubkey(), 5_000_000).await?;
    f.client.airdrop(f.student_b.pubkey(), 5_000_000).await?;

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
async fn test_assignment_checker(#[future] init_fixture: Result<Fixture>) {
    // @todo: add your happy path test scenario and the other test cases
    let f = init_fixture.await?;
    let course_pda = f.get_course_account().await?;
    assert_eq!(course_pda.authority, f.course_authority.pubkey());

    let batch_id: u16 = 1;
    let assignment_id: u16 = 1;
    let (assignment_checker_pda, _) = Pubkey::find_program_address(
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

    // Assignment: "Surname of the first man in space"
    let value_to_check = "Gagarin".to_string();

    let hash_chain_length = 10;
    // good enough for test
    let salt = [0; 32];

    let ground_truth_hash_chain_tail =
        Fixture::hash(hash_chain_length, &salt, value_to_check.as_bytes());

    assignment_checker_instruction::create(
        &f.client,
        batch_id,
        assignment_id,
        hash_chain_length,
        100,
        salt.clone(),
        ground_truth_hash_chain_tail,
        f.course_authority.pubkey().clone(),
        f.course_pda,
        assignment_checker_pda,
        f.system_program,
        [f.course_authority.clone()],
    )
    .await?;

    // successful check by student_a for the first time
    let check_result = f
        .check_assignment(
            f.student_a.clone(),
            assignment_checker_pda,
            f.course_pda,
            batch_id,
            assignment_id,
            value_to_check.as_bytes(),
            None,
        )
        .await?;

    assert_eq!(check_result.check_passed, true);
    assert_eq!(check_result.passed_first_time, true);

    // student_b tries to send the same hash value as student_b and fails the check
    // the chain became shorter and needs new hash
    let check_result = f
        .check_assignment(
            f.student_b.clone(),
            assignment_checker_pda,
            f.course_pda,
            batch_id,
            assignment_id,
            &[],
            Some(Fixture::hash(hash_chain_length - 1, &salt, &[])),
        )
        .await?;

    assert_eq!(check_result.check_passed, false);
    assert_eq!(check_result.passed_first_time, false);

    // student_a runs the check the second time
    // the second check marked as not passed the first time
    let check_result = f
        .check_assignment(
            f.student_a.clone(),
            assignment_checker_pda,
            f.course_pda,
            batch_id,
            assignment_id,
            value_to_check.as_bytes(),
            None,
        )
        .await?;

    assert_eq!(check_result.check_passed, true);
    assert_eq!(check_result.passed_first_time, false);
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

    #[throws]
    async fn get_checker_account(
        &self,
        checker_pda: Pubkey,
    ) -> assignment_checker::AssignmentChecker {
        self.client
            .account_data::<assignment_checker::AssignmentChecker>(checker_pda)
            .await?
    }

    #[throws]
    async fn get_check_result_account(
        &self,
        check_result_pda: Pubkey,
    ) -> assignment_checker::CheckResult {
        self.client
            .account_data::<assignment_checker::CheckResult>(check_result_pda)
            .await?
    }

    #[throws]
    async fn check_assignment(
        &self,
        student_keypair: Keypair,
        checker_pda: Pubkey,
        course_account: Pubkey,
        batch_id: u16,
        assignment_id: u16,
        value_to_check: &[u8],
        // to use custom hash instead of hasing value_to_check
        use_custom_hash_tail_parent: Option<[u8; 32]>,
    ) -> assignment_checker::CheckResult {
        let assignment_checker = self.get_checker_account(checker_pda).await?;
        let hash_chain_length = assignment_checker.hash_chain_length;
        let hash_chain_tail_parent = use_custom_hash_tail_parent.unwrap_or_else(|| {
            Self::hash(
                hash_chain_length - 1,
                &assignment_checker.salt,
                value_to_check,
            )
        });

        let (check_result_address, _) = Pubkey::find_program_address(
            &[
                assignment_checker::STUDENT_ACCOUNT_SEED,
                student_keypair.pubkey().as_ref(),
                assignment_checker::COURSE_ACCOUNT_SEED,
                course_account.as_ref(),
                assignment_checker::BATCH_ID_SEED,
                batch_id.to_be_bytes().as_ref(),
                assignment_checker::ASSIGNMENT_ID_SEED,
                assignment_id.to_be_bytes().as_ref(),
            ],
            &self.program.pubkey(),
        );
        assignment_checker_instruction::check(
            &self.client,
            batch_id,
            assignment_id,
            hash_chain_length,
            hash_chain_tail_parent,
            student_keypair.pubkey(),
            course_account,
            checker_pda,
            check_result_address,
            self.system_program,
            [student_keypair],
        )
        .await?;
        self.get_check_result_account(check_result_address).await?
    }

    fn hash(hash_chain_length: u16, salt: &[u8; 32], value_to_hash: &[u8]) -> [u8; 32] {
        assert!(hash_chain_length >= 2);
        let first_hash = blake3::hashv(&[salt, value_to_hash]);
        (0..hash_chain_length - 1)
            .fold(first_hash, |hash, _| blake3::hash(&hash.0))
            .0
    }
}
