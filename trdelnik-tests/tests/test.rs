use anchor_lang::solana_program::{blake3, sysvar::rent};
use anchor_lang::system_program;
use anchor_spl::associated_token::{self, get_associated_token_address};
use anchor_spl::token;
use fehler::throws;
use program_client::course_batch_manager_instruction;
use program_client::course_manager_instruction;
use trdelnik_client::{anyhow::Result, *};

#[throws]
#[fixture]
async fn start_course_batch() -> Fixture {
    let mut f = Fixture::new();
    // Deploy course manager, course batch manager and assignment checker programs
    f.deploy().await?;

    // Airdrop some lamports to the course authority and students A and B
    f.client
        .airdrop(f.course_authority.pubkey(), 10_000_000)
        .await?;
    f.client.airdrop(f.student_a.pubkey(), 5_000_000).await?;
    f.client.airdrop(f.student_b.pubkey(), 5_000_000).await?;

    // Course authority creates new course at the given the Program Derived Address
    f.course_pda = course_manager::course_canonical_pda(f.course_authority.pubkey(), &f.course_id);
    // create course account
    course_manager_instruction::create_new_course(
        &f.client,
        f.course_id,
        f.course_authority.pubkey(),
        f.course_pda,
        system_program::ID,
        [f.course_authority.clone()],
    )
    .await?;

    // Course authority creates new course batch at given the Program Derived Address
    f.course_batch_pda = course_batch_manager::batch_canonical_pda(f.course_pda, &f.batch_id);
    // Mint address for the course batch token
    f.course_batch_mint_pda =
        course_batch_manager::batch_mint_canonical_pda(f.course_pda, &f.batch_id);

    // create new course batch data and mint accounts
    course_batch_manager_instruction::create_new_batch(
        &f.client,
        f.batch_id,
        f.course_authority.pubkey(),
        f.course_pda,
        f.course_batch_pda,
        f.course_batch_mint_pda,
        system_program::ID,
        rent::id(),
        token::ID,
        [f.course_authority.clone()],
    )
    .await?;

    // enroll student_a into the batch and create course batch associated token account
    f.student_a_token_account =
        get_associated_token_address(&f.student_a.pubkey(), &f.course_batch_mint_pda);
    course_batch_manager_instruction::enroll_batch(
        &f.client,
        f.student_a.pubkey(),
        f.course_authority.pubkey(),
        f.course_batch_pda,
        f.course_batch_mint_pda,
        f.student_a_token_account,
        system_program::ID,
        token::ID,
        associated_token::ID,
        rent::id(),
        [f.student_a.clone()],
    )
    .await?;

    // enroll student_b into the batch and create course batch associated token account
    f.student_b_token_account =
        get_associated_token_address(&f.student_b.pubkey(), &f.course_batch_mint_pda);
    course_batch_manager_instruction::enroll_batch(
        &f.client,
        f.student_b.pubkey(),
        f.course_authority.pubkey(),
        f.course_batch_pda,
        f.course_batch_mint_pda,
        f.student_b_token_account,
        system_program::ID,
        token::ID,
        associated_token::ID,
        rent::id(),
        [f.student_b.clone()],
    )
    .await?;

    // Prepare assignment checker capable to check 10 - 1 students

    let ground_truth_hash_chain_tail = Fixture::hash(
        f.hash_chain_length,
        &f.salt,
        f.ground_truth_value.as_bytes(),
    );

    f.assignment_checker_pda =
        course_batch_manager::assignment_checker_canonical_pda(f.course_pda, &f.assignment_id);

    // create assignment checker
    course_batch_manager_instruction::create_assignment_checker(
        &f.client,
        f.assignment_id,
        f.hash_chain_length,
        100,
        f.salt.clone(),
        ground_truth_hash_chain_tail,
        f.course_authority.pubkey(),
        f.course_pda,
        f.assignment_checker_pda,
        assignment_checker::ID,
        course_batch_manager::ID,
        system_program::ID,
        [f.course_authority.clone()],
    )
    .await?;

    // init check result accounts for students A and B
    course_batch_manager_instruction::create_check_result(
        &f.client,
        f.assignment_id,
        f.student_a.pubkey(),
        f.course_pda,
        course_batch_manager::check_result_canonical_pda(
            f.student_a.pubkey(),
            f.course_pda,
            &f.assignment_id,
        ),
        assignment_checker::ID,
        course_batch_manager::ID,
        system_program::ID,
        [f.student_a.clone()],
    )
    .await?;
    course_batch_manager_instruction::create_check_result(
        &f.client,
        f.assignment_id,
        f.student_b.pubkey(),
        f.course_pda,
        course_batch_manager::check_result_canonical_pda(
            f.student_b.pubkey(),
            f.course_pda,
            &f.assignment_id,
        ),
        assignment_checker::ID,
        course_batch_manager::ID,
        system_program::ID,
        [f.student_b.clone()],
    )
    .await?;

    f
}

/// Test if student gets minted tokens once on the first passed assignment check
#[trdelnik_test]
async fn test_check_assignment(#[future] start_course_batch: Result<Fixture>) {
    let mut f = start_course_batch.await?;

    // saved authority in course and course batch data accounts
    let course_account = f.get_course_account().await?;
    assert_eq!(course_account.authority, f.course_authority.pubkey());
    let course_batch_account = f.get_course_batch_account().await?;
    assert_eq!(course_batch_account.authority, f.course_authority.pubkey());

    // empty balance for student_a
    let balance_a = f
        .client
        .get_token_balance(f.student_a_token_account)
        .await?;
    assert_eq!(balance_a.amount.as_str(), "0");

    // successful check by student_a for the first time
    let (student_a_hash_tail_parent, check_result) = f
        .check_assignment(
            f.student_a.clone(),
            f.student_a_token_account,
            f.assignment_checker_pda,
            f.course_pda,
            f.course_batch_pda,
            f.ground_truth_value.as_bytes(),
            None,
        )
        .await?;

    assert_eq!(check_result.check_passed, true);
    assert_eq!(check_result.passed_first_time, true);
    let balance_a = f
        .client
        .get_token_balance(f.student_a_token_account)
        .await?;
    assert_eq!(balance_a.amount.as_str(), "100");

    // student_b tries to send the same hash value as student_a and fails the check
    // the chain became shorter and needs new hash
    let (_, check_result) = f
        .check_assignment(
            f.student_b.clone(),
            f.student_b_token_account,
            f.assignment_checker_pda,
            f.course_pda,
            f.course_batch_pda,
            &[],
            Some(student_a_hash_tail_parent),
        )
        .await?;

    assert_eq!(check_result.check_passed, false);
    assert_eq!(check_result.passed_first_time, false);
    // empty balance for student_b
    let balance_b = f
        .client
        .get_token_balance(f.student_b_token_account)
        .await?;
    assert_eq!(balance_b.amount.as_str(), "0");

    // student_a runs the check the second time
    // the second check marked as not passed the first time
    let (_, check_result) = f
        .check_assignment(
            f.student_a.clone(),
            f.student_a_token_account,
            f.assignment_checker_pda,
            f.course_pda,
            f.course_batch_pda,
            f.ground_truth_value.as_bytes(),
            None,
        )
        .await?;

    assert_eq!(check_result.check_passed, true);
    assert_eq!(check_result.passed_first_time, false);
    // balance is not changed
    let balance_a = f
        .client
        .get_token_balance(f.student_a_token_account)
        .await?;
    assert_eq!(balance_a.amount.as_str(), "100");
}

/// Input keypairs / pubkeys / programs and data to configure tests
struct Fixture {
    client: Client,
    assignment_checker_program: Keypair,
    course_program: Keypair,
    course_batch_program: Keypair,

    course_authority: Keypair,
    course_id: [u8; 16],
    course_pda: Pubkey,

    batch_id: [u8; 16],
    course_batch_pda: Pubkey,
    course_batch_mint_pda: Pubkey,

    assignment_id: [u8; 16],
    ground_truth_value: String,
    hash_chain_length: u16,
    salt: [u8; 32],
    assignment_checker_pda: Pubkey,

    // can be related to course_authority
    student_a: Keypair,
    student_a_token_account: Pubkey,
    student_b: Keypair,
    student_b_token_account: Pubkey,
}

impl Fixture {
    fn new() -> Self {
        Fixture {
            client: Client::new(system_keypair(0)),
            assignment_checker_program: program_keypair(1),
            course_program: program_keypair(2),
            course_batch_program: program_keypair(3),
            course_authority: keypair(0),
            course_id: *b"web2_to_web3____",
            course_pda: Pubkey::default(),
            batch_id: *b"the_first_batch_",
            course_batch_pda: Pubkey::default(),
            course_batch_mint_pda: Pubkey::default(),
            assignment_id: *b"space_hero______",
            // Assignment: "Surname of the first man in space"
            ground_truth_value: "Gagarin".to_string(),
            // support up to 10 - 1 students
            hash_chain_length: 10,
            // good enough for test
            salt: [0; 32],
            assignment_checker_pda: Pubkey::default(),

            student_a: keypair(1),
            student_a_token_account: Pubkey::default(),
            student_b: keypair(2),
            student_b_token_account: Pubkey::default(),
        }
    }

    #[throws]
    async fn deploy(&mut self) {
        self.client
            .deploy_by_name(&self.course_program, "course_manager")
            .await?;
        self.client
            .deploy_by_name(&self.course_batch_program, "course_batch_manager")
            .await?;
        self.client
            .deploy_by_name(&self.assignment_checker_program, "assignment_checker")
            .await?;
    }

    #[throws]
    async fn get_course_account(&self) -> course_manager::Course {
        self.client
            .account_data::<course_manager::Course>(self.course_pda)
            .await?
    }

    #[throws]
    async fn get_course_batch_account(&self) -> course_batch_manager::CourseBatch {
        self.client
            .account_data::<course_batch_manager::CourseBatch>(self.course_batch_pda)
            .await?
    }

    #[throws]
    async fn get_checker_account(
        &self,
        checker_pda: Pubkey,
    ) -> assignment_checker::AssignmentCheckerState {
        self.client
            .account_data::<course_batch_manager::AssignmentCheckerState>(checker_pda)
            .await?
    }

    #[throws]
    async fn get_check_result_account(
        &self,
        check_result_pda: Pubkey,
    ) -> assignment_checker::CheckResult {
        self.client
            .account_data::<course_batch_manager::CheckResult>(check_result_pda)
            .await?
    }

    /// Checks assignment and returns the hashed value_to_check and the result of the check
    #[throws]
    async fn check_assignment(
        &self,
        student_keypair: Keypair,
        student_token_address: Pubkey,
        checker_data_address: Pubkey,
        course_data_address: Pubkey,
        course_batch_address: Pubkey,
        value_to_check: &[u8],
        // to use custom hash instead of hasing value_to_check
        use_custom_hash_tail_parent: Option<[u8; 32]>,
        // (hash used for check, check_result)
    ) -> ([u8; 32], course_batch_manager::CheckResult) {
        let assignment_checker = self.get_checker_account(checker_data_address).await?;
        let hash_chain_length = assignment_checker.hash_chain_length;
        let hash_chain_tail_parent = use_custom_hash_tail_parent.unwrap_or_else(|| {
            Self::hash(
                hash_chain_length - 1,
                &assignment_checker.salt,
                value_to_check,
            )
        });

        let check_result_address = course_batch_manager::check_result_canonical_pda(
            student_keypair.pubkey(),
            course_data_address,
            &assignment_checker.assignment_id,
        );

        course_batch_manager_instruction::check_assignment(
            &self.client,
            hash_chain_length,
            hash_chain_tail_parent,
            student_keypair.pubkey(),
            course_data_address,
            course_batch_address,
            checker_data_address,
            check_result_address,
            self.course_batch_mint_pda,
            student_token_address,
            system_program::ID,
            token::ID,
            assignment_checker::ID,
            course_batch_manager::ID,
            [student_keypair],
        )
        .await?;
        (
            hash_chain_tail_parent,
            self.get_check_result_account(check_result_address).await?,
        )
    }

    fn hash(hash_chain_length: u16, salt: &[u8; 32], value_to_hash: &[u8]) -> [u8; 32] {
        assert!(hash_chain_length >= 2);
        let first_hash = blake3::hashv(&[salt, value_to_hash]);
        (0..hash_chain_length - 1)
            .fold(first_hash, |hash, _| blake3::hash(&hash.0))
            .0
    }
}
