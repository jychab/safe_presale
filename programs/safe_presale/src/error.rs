use anchor_lang::prelude::*;

#[error_code]
pub enum CustomError {
    #[msg("Nft is not non-fungible")]
    NftIsNotNonFungible,
    #[msg("Unable to claim as presale is still ongoing")]
    PresaleIsStillOngoing,
    #[msg("Presale has ended!")]
    PresaleHasEnded,
    #[msg("Token already launched")]
    TokenHasLaunched,
    #[msg("Conversion to u64 failed with an overflow or underflow")]
    ConversionFailure,
    #[msg("Integer Overflow Error")]
    IntegerOverflow,
    #[msg("Mint not allowed in this pool")]
    MintNotAllowedInPool,
    #[msg("Mint metadata is owned by the incorrect program")]
    InvalidMintMetadataOwner,
    #[msg("Invalid mint metadata")]
    InvalidMintMetadata,
    #[msg("Mint not allowed to claim")]
    MintNotAllowedToClaim,
    #[msg("Invalid pool to claim")]
    InvalidPool,
    #[msg("Invalid reward mint")]
    InvalidRewardMint,
    #[msg("Nothing left to claim")]
    MaximumAmountClaimed,
    #[msg("Vesting Period has not ended")]
    VestingStillInProgress,
}
