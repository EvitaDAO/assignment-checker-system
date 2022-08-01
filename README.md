Automated assignment checker system
===================================


This project is a result of practice to understand main Solana concepts like
`owner`, `authority`, `Program Derived Addresses (PDA)`, `signers`, `PDA signers`, `cross-program invocation (CPI)`, `Associated token account (ATA)`. It contains a prototype of on-chain `automated assignment checker system (AACS)` that mints preconfigured number of course batch tokens when a student successfuly solves some assignment and sends correct result.

Solana is chosen because it allows to build energy efficient and concurrent on-chain programs. Due to efficiency transaction cost and confirmation time is much lower than in other public blockchains. Both on-chain and client side of the programs can benefit from usage of Rust and its ecosystem.

The prototype uses [Anchor](https://github.com/coral-xyz/anchor) framework to organize accounts and their validation. [Trdelnik](https://github.com/Ackee-Blockchain/trdelnik) is used to generate a test client and write a command line test that spins up local validator, deploys programs, initializes test fixture with configured accounts, creates them using the program instructions and runs the logic that tests behaviour of the `AACS` prototype.

Roles and programs
------------------

* `Course authority` - organizes `Courses` and their content. Promotes `Courses` to `Students` and runs `Course batches`. Prepares assignments for `Students`. Anybody can be a `Course authority` and organize own `Courses`.

* `Student` - enrolls into `Course batch`, solves assignments and sends hashed solutions to get `Course batch tokens`. Fully implemented MVP of AACS could exchange these batch specific tokens into other assets (like certifications or hiring rating). This functionality is outside of the prototype scope.

* `CourseManager` program derives and owns `Course` account. This account keeps `Course authority` pubkey for validation purposes. The account address (which is PDA) is used as a namespace to derive addresses of other accounts like `Course batch`, `Course batch Mint`, `Assignment checker`.

* `CourseBatchManager` program

    * derives and owns `Course batch` and `Course batch Mint` accounts for each batch created by the `Course Authority`. `Course batch` account is the mint authority of `Course batch Mint` account. Both accounts have PDAs. Only `CourseBatchManager` could sign for them.
    * `Students` can enroll in the batch and get their `Student course batch ATA` with zero balance of `Course batch tokens`.
    * `Course authority` can create `AssignmentCheckerState` accounts for each `Course` assignment, provide ground truth solution hashes and configure number of tokens that will be minted and tranfered to `Student course batch ATA` when correct solution is provided by a `Student`
    * `Students` can start solving assignments and create `CheckResult` accounts for them. `CheckResult` answers on two questions:

        1. whether the assignment check has ever passed
        2. whether the check has passed for the first time

        On finding a potential assignment solution `Student` initiates `CheckAssignment` operation. If it's succeded `Student` receives the tokens awarded for the submission of the correct solution hash. Tokens are awarded for each `Student` only once per `Course` and assignment.
* `AssignmentChecker` program is an owner of `AssignmentCheckerState` and `CheckResult` accounts. It checks whether provided solution hash with the given `expected_hash_chain_length` correctly hashes into stored `ground_truth_hash_chain_tail`. On successful check it cuts the tail of the [hash chain](https://en.wikipedia.org/wiki/Hash_chain) and removes an opportunity to try the same solution hash by another student acting like a sequence of one-time passwords. `AssignmentChecker` stores the status of the check in `CheckResult` account.

    * `AssignmentCheckerState` and `CheckResult` accounts are PDAs derived from parameterized `result_processor_program` and required to be transaction signers
        * only the `result_processor_program` can create these accounts and run assignment solution check
        * `CourseBatchManager` program creates these accounts for `AssignmentChecker` and sets it be the accounts owner. It initializes them by doing `CPI` calls to `AssignmentChecker` - the only program that can mutate them.
        * `CourseBatchManager` program plays the role of `result_processor_program` during `check_assignment` operations. It does `CPI` call to `AssignmentChecker` to do the actual check and analyzes the state of `CheckResult` account
        * another program cannot pass `AssignmentCheckerState` and `CheckResult` accounts derived from `CourseBatchManager` because it cannot sign for these PDAs.
    * `AssignmentChecker` returns custom program errors when a hash chain has run out of capacity or `check_assignment` is called with incorrect `expected_hash_chain_length`. The later error could happen during concurrent checks made by several students. Client is expected to retry the call with updated `expected_hash_chain_length` value.

Testing
-------

0. [Install](https://github.com/Ackee-Blockchain/trdelnik#dependencies) Rust + Cargo, Solana, Anchor and Trdelnik.

1. Build shared objects for on-chain programs, generate their Anchor IDL and Trdelink test `.program_client` module

        trdelnik build

2. Build and run [test](./trdelnik-tests/tests/test.rs#L160) that will setup and test the entire `check_assignment` flow.

        trdelnik test

3. Optionally during test execution you can monitor program logs from the local test validator

        solana logs -u localhost

    When tokens are minted the following line is logged:

        Program log: minted 100 tokens to Too1UPuAw5enEA4PkdZUDnPfye9TsqH5bsqAWmYCas7
