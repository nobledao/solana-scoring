//! Scoring program state, recording on-chain metadata for each scoring system.
use {
    borsh::{
        BorshDeserialize,
        // BorshSchema,
        BorshSerialize,
    },
    solana_program::{
        // program_option::COption,
        // program_pack::IsInitialized,
        pubkey::Pubkey,
    },
};

/// Scoring Mint data, supporting on-chain programs issuing points and client
/// applications that render wallet scores.
// #[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, PartialEq)]
pub struct Mint {
    /// Authority used to issue or slash points. The mint authority may only be
    /// set during mint creation.
    pub score_authority: Pubkey,
    /// Optional authority to freeze all scores globally. Used for time-limited
    /// games or events. Freezing the mint supports creating final leaderboards.
    /// May not be modified after creating the mint.
    pub freeze_authority: Option<Pubkey>,
    /// Lifecycle state for the mint.
    pub state: MintState,
    /// URI for JSON metadata describing this mint's points. Maximum length is
    /// 128 bytes. Expected format is the metaplex format:
    /// https://docs.metaplex.com/nft-standard#uri-json-schema
    pub metadata_uri: String,
}

impl Mint {
    /// Maximum size of the data in a Scoring mint account.
    pub const SIZE : usize = 32 + 33 + 1 + 128;
}

// impl Sealed for Mint {}
// impl IsInitialized for Mint {
//     fn is_initialized(&self) -> bool {
//         self.state == MintState::Initialized || self.state == MintState::Frozen
//     }
// }

/// Mint state.
#[derive(BorshSerialize, BorshDeserialize, Clone, Copy, Debug, PartialEq)]
pub enum MintState {
    /// Mint is not yet initialized
    Uninitialized,
    /// Mint is initialized and the mint authority may issue points at any time.
    Initialized,
    /// Mint has been frozen by the mint freeze authority and points may not be
    /// issued in the future.
    Frozen,
}
