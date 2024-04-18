use anchor_lang::{prelude::*, solana_program::program_pack::IsInitialized};

pub mod public_keys {
    pub mod amm_v4_mainnet {
        anchor_lang::declare_id!("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8");
    }
    pub mod amm_v4_devnet {
        anchor_lang::declare_id!("HWy1jotHpo6UqeQxx49dpYYdQB8wj9Qk9MdxwjLvDHB8");
    }
    pub mod clmm_mainnet {
        anchor_lang::declare_id!("CAMMCzo5YL8w4VFF8KVHrK22GGUsp5VTaW7grrKgrWqK");
    }
    pub mod clmm_devnet {
        anchor_lang::declare_id!("devi51mZmdwUJGU9hjN27vEz64Gps7uUefqxg27EAtH");
    }
    pub mod wsol {
        anchor_lang::declare_id!("So11111111111111111111111111111111111111112");
    }
    pub mod fee_collector {
        anchor_lang::declare_id!("73hCTYpoZNdFiwbh2PrW99ykAyNcQVfUwPMUhu9ogNTg");
    }
}

pub const MINT_PREFIX: &str = "mint";

pub const GRACE_PERIOD: i64 = 7 * 24 * 60 * 60;

#[account]
pub struct Pool {
    pub bump: u8,
    pub requires_collection: bool,
    pub delegate: Option<Pubkey>,
    pub authority: Pubkey,
    pub mint: Pubkey,
    pub lp_mint: Option<Pubkey>,
    pub lp_mint_supply: Option<u64>,
    pub liquidity_collected: u64,
    pub max_amount_per_purchase: Option<u64>,
    pub creator_fee_basis_points: u16,
    pub total_supply: u64,
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
    pub delegate: Option<Pubkey>, // for automating launching of pools
    pub authority: Pubkey,        // creator of the pool
    pub pool: Pubkey,
    pub mint: Pubkey,
    pub presale_target: u64, // amount the creator will receive from the presale
    pub presale_time_limit: i64, // presale ending time
    pub creator_fee_basis_points: u16, // percentage of the lp tokens that the creator will receive
    pub total_supply: u64,
    pub decimal: u8,
    pub vesting_period: u32, // vesting period for the lp tokens
    pub max_amount_per_purchase: Option<u64>,
    pub requires_collection: bool,
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
    pub lp_elligible: u64, // this amount is including creators fee
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
    pub amount_wsol_withdrawn: u64,
    pub original_mint_owner: Pubkey,
}
