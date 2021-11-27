//! Program instructions

use crate::{check_program_account};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    program_error::ProgramError,
    pubkey::Pubkey,
};

/// Instructions supported by the scoring program.
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, PartialEq)]
pub enum ScoreInstruction {
    /// Create a new score type's mint.
    ///
    /// The `InitializeScoreMint` instruction requires no signers and MUST be
    /// included within the same Transaction as the system program's
    /// `CreateAccount` instruction that creates the account being initialized.
    /// Otherwise another party can acquire ownership of the uninitialized
    /// account.
    ///
    /// Accounts expected by this instruction:
    ///
    ///   0. `[writable]` The scoring mint to initialize.
    ///
    InitializeScoreMint{
        /// The authority to issue or slash an accounts score.
        score_authority: Pubkey,
        /// The freeze authority of the scoring mint.
        freeze_authority: Option<Pubkey>,
        /// The URI to JSON metadata for the score type.
        metadata_uri: String,
    },
}

/// Creates a `InitializeScoreMint` instruction.
pub fn initialize_score_mint(
    scoring_program_id: &Pubkey,
    mint_pubkey: &Pubkey,
    score_authority_pubkey: &Pubkey,
    freeze_authority_pubkey: Option<&Pubkey>,
    metadata_uri: String,
) -> Result<Instruction, ProgramError> {
    check_program_account(scoring_program_id)?;
    let freeze_authority = freeze_authority_pubkey.cloned().into();
    let data = ScoreInstruction::InitializeScoreMint {
        score_authority: *score_authority_pubkey,
        freeze_authority,
        metadata_uri,
    }
    .try_to_vec().unwrap();

    let accounts = vec![AccountMeta::new(*mint_pubkey, false)];

    Ok(Instruction {
        program_id: *scoring_program_id,
        accounts,
        data,
    })
}