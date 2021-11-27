//! Error types

use num_derive::FromPrimitive;
use solana_program::{decode_error::DecodeError, program_error::ProgramError};
use thiserror::Error;

/// Errors that may be returned by the program.
#[derive(Clone, Debug, Eq, Error, FromPrimitive, PartialEq)]
pub enum ScoreError {
    /// Incorrect authority provided on update or freeze
    #[error("Incorrect authority provided on update or freeze")]
    IncorrectAuthority,

    /// Data type mismatched
    #[error("Data type length mismatched")]
    DataTypeMismatch,

    /// The mint exists and cannot be re-initialized.
    #[error("Mint exists")]
    MintExists,

    /// Scoring mint account is not rent-exempt as required.
    #[error("Scoring mint account must hold enough lamports to be rent-exempt")]
    ScoringMintNotRentExempt,
}
impl From<ScoreError> for ProgramError {
    fn from(e: ScoreError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
impl<T> DecodeError<T> for ScoreError {
    fn type_of() -> &'static str {
        "Score Error"
    }
}