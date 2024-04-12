use anchor_lang::prelude::*;

#[error_code]
pub enum CustomError {
    #[msg("Nft must be Non-Fungible")]
    NftIsNotNonFungible,
    #[msg("Presale is still ongoing")]
    PresaleIsStillOngoing,
    #[msg("Presale has ended")]
    PresaleHasEnded,
    #[msg("Presale target not met")]
    PresaleTargetNotMet,
    #[msg("Token already launched")]
    TokenHasLaunched,
    #[msg("Creator failed to launch project within 7 days.")]
    PoolHasExpired,
    #[msg("Presale target is met, Unable to withdraw. ")]
    WaitingForCreatorToLaunch,
    #[msg("Conversion to u64 failed with an overflow or underflow")]
    ConversionFailure,
    #[msg("Integer Overflow Error")]
    IntegerOverflow,
    #[msg("Number cannot be negative")]
    InvalidNegativeValue,
    #[msg("Mint not allowed in this pool")]
    MintNotAllowedInPool,
    #[msg("Mint metadata is owned by the incorrect program")]
    InvalidMintMetadataOwner,
    #[msg("Invalid mint metadata")]
    InvalidMintMetadata,
    #[msg("Invalid Mint")]
    MintNotAllowed,
    #[msg("Invalid pool to claim")]
    InvalidPool,
    #[msg("Invalid reward mint")]
    InvalidRewardMint,
    #[msg("Invalid lp mint")]
    InvalidLpMint,
    #[msg("Nothing left to claim")]
    MaximumAmountClaimed,
    #[msg("Either presale or vesting is still ongoing")]
    UnauthorizedAtCurrentTime,
    #[msg("Vesting Supply cannot be larger than Total Supply")]
    VestingSupplyLargerThanTotalSupply,
    #[msg("Creator Fees Basis Points cannot exceed 10000")]
    CreatorBasisPointsExceedMaximumAmount,
    #[msg("Amount cannot be zero")]
    NumberCannotBeZero,
    #[msg("Purchase amount exceeded")]
    AmountPurchaseExceeded,
    #[msg("Check elligibility first")]
    CheckClaimFirstBeforeClaiming,
    #[msg("Already checked")]
    ClaimedAlreadyChecked,
    #[msg("Signer must be owner of nft")]
    InvalidSigner,
    #[msg("Purchase authorisation record is missing")]
    PurchaseAuthorisationRecordMissing,
    #[msg("Collection is not authorised")]
    UnauthorisedCollection,
}
