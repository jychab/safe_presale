use anchor_lang::prelude::*;

#[error_code]
pub enum CustomError {
    #[msg("Nft is not non-fungible")]
    NftIsNotNonFungible,
    #[msg("Unable to claim as presale is still ongoing")]
    PresaleIsStillOngoing,
    #[msg("Presale has ended!")]
    PresaleHasEnded,
    #[msg("Exceeded presale time limit")]
    PresaleTimeLimtExceeded,
    #[msg("Presale target not met!")]
    PresaleTargetNotMet,
    #[msg("Token already launched")]
    TokenHasLaunched,
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
    #[msg("Nothing left to claim")]
    MaximumAmountClaimed,
    #[msg("Either presale time limit has not ended or Vesting is still in progress")]
    UnauthorizedAtCurrentTime,
    #[msg("Vesting Supply larget than total supply")]
    VestingSupplyLargerThanTotalSupply,
    #[msg("Creator Fees basis points exceed 10000")]
    CreatorBasisPointsExceedMaximumAmount,
    #[msg("Purchase Amount cannot be zero")]
    AmountPurchasedIsZero,
    #[msg("Check Claim Amount first before claiming")]
    CheckClaimFirstBeforeClaiming,
    #[msg("Claim Amount is already updated")]
    ClaimedAlreadyChecked,
}
