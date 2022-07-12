// DO NOT EDIT - automatically generated file
pub mod assignmentchecker_instruction {
    use trdelnik_client::*;
    pub static PROGRAM_ID: Pubkey = Pubkey::new_from_array([
        87u8, 89u8, 158u8, 244u8, 178u8, 110u8, 21u8, 1u8, 112u8, 202u8, 133u8, 68u8, 245u8, 15u8,
        230u8, 208u8, 14u8, 108u8, 110u8, 30u8, 247u8, 166u8, 115u8, 174u8, 121u8, 206u8, 204u8,
        232u8, 1u8, 104u8, 218u8, 128u8,
    ]);
    pub async fn create(
        client: &Client,
        i_batch_id: u16,
        i_assignment_id: u16,
        i_to_mint_on_successful_check: u16,
        i_hash_chain_length: u16,
        i_salt: [u8; 32],
        i_ground_truth_hash_chain_tail: [u8; 32],
        a_course_authority: anchor_lang::solana_program::pubkey::Pubkey,
        signers: impl IntoIterator<Item = Keypair> + Send + 'static,
    ) -> Result<EncodedConfirmedTransaction, ClientError> {
        Ok(client
            .send_instruction(
                PROGRAM_ID,
                assignmentchecker::instruction::Create {
                    batch_id: i_batch_id,
                    assignment_id: i_assignment_id,
                    to_mint_on_successful_check: i_to_mint_on_successful_check,
                    hash_chain_length: i_hash_chain_length,
                    salt: i_salt,
                    ground_truth_hash_chain_tail: i_ground_truth_hash_chain_tail,
                },
                assignmentchecker::accounts::Create {
                    course_authority: a_course_authority,
                },
                signers,
            )
            .await?)
    }
    pub fn create_ix(
        i_batch_id: u16,
        i_assignment_id: u16,
        i_to_mint_on_successful_check: u16,
        i_hash_chain_length: u16,
        i_salt: [u8; 32],
        i_ground_truth_hash_chain_tail: [u8; 32],
        a_course_authority: anchor_lang::solana_program::pubkey::Pubkey,
    ) -> Instruction {
        Instruction {
            program_id: PROGRAM_ID,
            data: assignmentchecker::instruction::Create {
                batch_id: i_batch_id,
                assignment_id: i_assignment_id,
                to_mint_on_successful_check: i_to_mint_on_successful_check,
                hash_chain_length: i_hash_chain_length,
                salt: i_salt,
                ground_truth_hash_chain_tail: i_ground_truth_hash_chain_tail,
            }
            .data(),
            accounts: assignmentchecker::accounts::Create {
                course_authority: a_course_authority,
            }
            .to_account_metas(None),
        }
    }
}
