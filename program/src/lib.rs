//! Solana scoring program
#![deny(missing_docs)]

mod entrypoint;
pub mod error;
pub mod instruction;
pub mod processor;
pub mod state;
pub mod utils;

// Export current sdk types for downstream users building with a different sdk version
pub use solana_program;
use solana_program::{entrypoint::ProgramResult, program_error::ProgramError, pubkey::Pubkey};

solana_program::declare_id!("SCorEKFKYJud973vCJvWFphgqQGAHo9Ruxuf622LER1");

/// Checks that the supplied program ID is the correct one for scoring.
pub fn check_program_account(solana_program_id: &Pubkey) -> ProgramResult {
    if solana_program_id != &id() {
        return Err(ProgramError::IncorrectProgramId);
    }
    Ok(())
}