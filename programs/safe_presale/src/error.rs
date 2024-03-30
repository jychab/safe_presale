use anchor_lang::prelude::*;

#[error_code]
pub enum CustomError {
    #[msg("Unable to claim as presale is still ongoing!")]
    PresaleIsStillOngoing,

    #[msg("Original mint is invalid")]
    InvalidOriginalMint,
    #[msg("Token Manager mint is invalid")]
    InvalidTokenManagerMint,
    #[msg("Invalid user original mint token account")]
    InvalidUserOriginalMintTokenAccount,
    #[msg("Invalid user token manager mint account")]
    InvalidUserMintTokenAccount,
    #[msg("Invalid stake entry original mint token account")]
    InvalidStakeEntryOriginalMintTokenAccount,
    #[msg("Invalid stake entry token manager mint token account")]
    InvalidStakeEntryMintTokenAccount,
    #[msg("Invalid unstake user only last staker can unstake")]
    InvalidUnstakeUser,
    #[msg("Invalid stake pool")]
    InvalidStakePool,
    #[msg("No mint metadata")]
    NoMintMetadata,
    #[msg("Mint not allowed in this pool")]
    MintNotAllowedInPool,
    #[msg("Invalid stake pool authority")]
    InvalidPoolAuthority,
    #[msg("Invalid stake type")]
    InvalidStakeType,
    #[msg("Invalid stake entry stake token account")]
    InvalidStakeEntryStakeTokenAccount,
    #[msg("Invalid last staker")]
    InvalidLastStaker,
    #[msg("Invalid token manager program")]
    InvalidTokenManagerProgram,
    #[msg("Invalid receipt mint")]
    InvalidReceiptMint,
    #[msg("Stake entry already has tokens staked")]
    StakeEntryAlreadyStaked,
    #[msg("Invalid authority")]
    InvalidAuthority,
    #[msg("Cannot close staked entry")]
    CannotCloseStakedEntry,
    #[msg("Cannot close staked entry")]
    CannotClosePoolWithStakedEntries,
    #[msg("Token still has some cooldown seconds remaining")]
    CooldownSecondRemaining,
    #[msg("Minimum stake seconds not satisfied")]
    MinStakeSecondsNotSatisfied,
    #[msg("Invalid stake authorization provided")]
    InvalidStakeAuthorizationRecord,
    #[msg("Invalid mint metadata")]
    InvalidMintMetadata,
    #[msg("Stake pool has ended")]
    StakePoolHasEnded,
    #[msg("Mint metadata is owned by the incorrect program")]
    InvalidMintMetadataOwner,
    #[msg("Stake mint already intialized")]
    StakeMintAlreadyInitialized,
    #[msg("Invalid stake entry")]
    InvalidStakeEntry,
    #[msg("Cannot update unstaked entry")]
    CannotUpdateUnstakedEntry,

    //reward distribution error
    #[msg("Invalid token account")]
    InvalidTokenAccount,
    #[msg("Invalid reward mint")]
    InvalidRewardMint,
    #[msg("Invalid user reward mint token account")]
    InvalidUserRewardMintTokenAccount,
    #[msg("Invalid reward distributor")]
    InvalidRewardDistributor,
    #[msg("Invalid reward distributor authority")]
    InvalidRewardDistributorAuthority,
    #[msg("Invalid reward distributor kind")]
    InvalidRewardDistributorKind,
    #[msg("Initial supply required for kind treasury")]
    SupplyRequired,
    #[msg("Invalid authority")]
    InvalidPoolDistributor,
    #[msg("Distributor is already open")]
    DistributorNotClosed,
    #[msg("Distributor is already closed")]
    DistributorAlreadyClosed,
    #[msg("Invalid stake entry")]
    InvalidRewardEntry,
    #[msg("Invalid reward distributor token account")]
    InvalidRewardDistributorTokenAccount,
    #[msg("Invalid authority token account")]
    InvalidAuthorityTokenAccount,
    #[msg("Invalid payer")]
    InvalidPayer,
    #[msg("Max reward seconds claimed")]
    MaxRewardSecondsClaimed,
    #[msg("Invalid reward token account owner")]
    InvalidRewardTokenOwner,
    #[msg("Invalid self transfer")]
    InvalidSelfTransfer,
    #[msg("Not enough reward tokens")]
    NotEnoughRewardTokens,
    #[msg("Invalid instruction")]
    InvalidInstruction,
}
