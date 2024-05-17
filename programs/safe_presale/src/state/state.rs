use anchor_lang::{prelude::*, solana_program::program_pack::IsInitialized};

pub const FEE_COLLECTOR: &str = "73hCTYpoZNdFiwbh2PrW99ykAyNcQVfUwPMUhu9ogNTg";

pub const RAYDIUM_CPMM_V4_DEVNET: &str = "CPMDWBwJDtYax9qW7AyRuVC19Cc4L4Vcy4n2BHAbHkCW";

pub const RAYDIUM_CPMM_V4_MAINNET: &str = "CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C";

pub const MINT_PREFIX: &str = "mint";

pub const GRACE_PERIOD: i64 = 7 * 24 * 60 * 60;

#[account]
pub struct Pool {
    pub bump: u8,
    pub quote_mint: Pubkey,
    pub requires_collection: bool,
    pub delegate: Option<Pubkey>,
    pub authority: Pubkey,
    pub mint: Pubkey,
    pub lp_mint: Option<Pubkey>,
    pub lp_mint_supply: Option<u64>,
    pub lp_mint_supply_for_creator: Option<u64>,
    pub lp_mint_claimed_by_creator: u64,
    pub lp_mint_last_claimed_by_creator: Option<i64>,
    pub liquidity_collected: u64,
    pub max_amount_per_purchase: Option<u64>,
    pub creator_fee_basis_points: u16,
    pub liquidity_pool_supply: u64,
    pub initial_supply: u64,
    pub initial_supply_for_creator: u64,
    pub presale_target: u64,
    pub presale_time_limit: i64,
    pub vesting_period: u32,
    pub vesting_started_at: Option<i64>,
}
pub const POOL_PREFIX: &str = "pool";
pub const POOL_SIZE: usize = std::mem::size_of::<Pool>() + 8;

#[account]
pub struct PurchaseAuthorisationRecord {
    pub pool: Pubkey,
    pub collection_mint: Pubkey,
    pub bump: u8,
}
pub const PURCHASE_AUTHORISATION_PREFIX: &str = "authorisation";
pub const PURCHASE_AUTHORISATION_SIZE: usize =
    std::mem::size_of::<PurchaseAuthorisationRecord>() + 8;

#[account]
pub struct PurchaseReceipt {
    pub is_initialized: bool,
    pub bump: u8,
    pub pool: Pubkey,
    pub amount: u64,
    pub mint_elligible: Option<u64>,
    pub mint_claimed: bool,
    pub lp_elligible: Option<u64>,
    pub original_mint: Pubkey,
    pub lp_claimed: u64,
    pub last_claimed_at: Option<i64>,
}

impl IsInitialized for PurchaseReceipt {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

pub const PURCHASE_RECEIPT_PREFIX: &str = "receipt";
pub const PURCHASE_RECEIPT_SIZE: usize = std::mem::size_of::<PurchaseReceipt>() + 8;

#[event]
pub struct InitializedPoolEvent {
    pub delegate: Option<Pubkey>,
    pub authority: Pubkey,
    pub pool: Pubkey,
    pub mint: Pubkey,
    pub presale_target: u64,
    pub presale_time_limit: i64,
    pub creator_fee_basis_points: u16,
    pub liquidity_pool_supply: u64,
    pub initial_supply: u64,
    pub initial_supply_for_creator: u64,
    pub decimal: u8,
    pub vesting_period: u32,
    pub max_amount_per_purchase: Option<u64>,
    pub requires_collection: bool,
    pub quote_mint: Pubkey,
}

#[event]
pub struct CreatePurchaseAuthorisationEvent {
    pub payer: Pubkey,
    pub collection_mint: Pubkey,
    pub pool: Pubkey,
}

#[event]
pub struct PurchasedPresaleEvent {
    pub payer: Pubkey,
    pub amount: u64,
    pub pool: Pubkey,
    pub original_mint: Pubkey,
}

#[event]
pub struct CheckClaimEvent {
    pub payer: Pubkey,
    pub pool: Pubkey,
    pub original_mint: Pubkey,
    pub lp_elligible: u64,
    pub mint_elligible: u64,
}

#[event]
pub struct ClaimRewardEvent {
    pub payer: Pubkey,
    pub pool: Pubkey,
    pub original_mint: Pubkey,
    pub original_mint_owner: Pubkey,
    pub mint_elligible: u64,
}

#[event]
pub struct ClaimRewardForCreatorEvent {
    pub payer: Pubkey,
    pub pool: Pubkey,
    pub mint_elligible: u64,
}

#[event]
pub struct WithdrawLpTokenForCreatorEvent {
    pub payer: Pubkey,
    pub pool: Pubkey,
    pub lp_claimed: u64,
    pub last_claimed_at: i64,
}

#[event]
pub struct WithdrawLpTokenEvent {
    pub payer: Pubkey,
    pub pool: Pubkey,
    pub lp_claimed: u64,
    pub last_claimed_at: i64,
    pub original_mint: Pubkey,
    pub original_mint_owner: Pubkey,
}

#[event]
pub struct LaunchTokenAmmEvent {
    pub payer: Pubkey,
    pub pool: Pubkey,
    pub amount_coin: u64,
    pub amount_pc: u64,
    pub amount_lp_received: u64,
    pub lp_mint: Pubkey,
    pub vesting_started_at: i64,
}

#[event]
pub struct WithdrawEvent {
    pub payer: Pubkey,
    pub pool: Pubkey,
    pub original_mint: Pubkey,
    pub amount_withdrawn: u64,
    pub original_mint_owner: Pubkey,
}
