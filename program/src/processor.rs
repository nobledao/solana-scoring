//! Program state processor

use {
    crate::{
        error::ScoreError, instruction::ScoreInstruction, state::Mint, state::MintState,
        utils::try_from_slice_checked,
    },
    borsh::{BorshDeserialize, BorshSerialize},
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        entrypoint::ProgramResult,
        msg,
        // program::invoke_signed,
        program_error::ProgramError,
        pubkey::Pubkey,
        rent::Rent,
        // system_instruction,
        // system_program,
        sysvar::Sysvar, // for Rent::get()
    },
};

/// Instruction processor
pub fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    input: &[u8],
) -> ProgramResult {
    let instruction = ScoreInstruction::try_from_slice(input).map_err(|e| {
        msg!("input: {}", input.len());
        msg!("input: {:x?}", input);
        msg!("failed to unpack ScoreInstruction instruction: {}", e);
        ProgramError::InvalidInstructionData
    })?;
    match instruction {
        ScoreInstruction::InitializeScoreMint {
            score_authority,
            freeze_authority,
            metadata_uri,
        } => process_initialize_score_mint(
            _program_id,
            accounts,
            &score_authority,
            freeze_authority,
            metadata_uri,
        ),
    }
}

fn process_initialize_score_mint(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    score_authority: &Pubkey,
    freeze_authority: Option<Pubkey>,
    metadata_uri: String,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let mint_info = next_account_info(account_info_iter)?;
    let mint_data_len = mint_info.data_len();
    let rent = Rent::get()?;

    // Check the mint account data - should not yet be initialized.
    let mut mintdata = try_from_slice_checked::<Mint>(&mint_info.data.borrow(), Mint::SIZE)?;
    if mintdata.state != MintState::Uninitialized {
        return Err(ScoreError::MintExists.into());
    }
    if !rent.is_exempt(mint_info.lamports(), mint_data_len) {
        return Err(ScoreError::ScoringMintNotRentExempt.into());
    }
    // Update mint fields. Owner check is implicit: if owner != crate::id(), then writes are rejected.
    mintdata.score_authority = *score_authority;
    mintdata.freeze_authority = freeze_authority;
    mintdata.state = MintState::Initialized;
    mintdata.metadata_uri = metadata_uri;

    mintdata
        .serialize(&mut *mint_info.data.borrow_mut())
        .map_err(|e| e.into())
}
