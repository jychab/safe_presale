use anchor_lang::prelude::*;

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
}

#[account]
pub struct Pool {
    pub bump: u8,
    pub allow_purchase: bool,
    pub identifier: u64,
    pub authority: Pubkey,
    pub requires_collections: Vec<Pubkey>,
    pub mint: Pubkey,
    pub liquidity_collected: u64,
    pub vested_supply: u64,
    pub total_supply: u64,
    pub vesting_period: i64,
    pub vesting_started_at: Option<i64>,
    pub vesting_period_end: Option<i64>,
}
pub const POOL_PREFIX: &str = "pool";
pub const POOL_SIZE: usize = 8 + std::mem::size_of::<Pool>() + 8;

#[account]
pub struct Identifier {
    pub bump: u8,
    pub count: u64,
}

pub const IDENTIFIER_PREFIX: &str = "identifier";
pub const IDENTIFIER_SIZE: usize = 8 + std::mem::size_of::<Identifier>() + 8;

#[account]
pub struct PurchaseReceipt {
    pub bump: u8,
    pub pool: Pubkey,
    pub amount: u64,
    pub original_mint: Pubkey,
    pub mint_claimed: u64,
    pub mint_elligible: Option<u64>,
    pub last_claimed_at: Option<i64>,
}

pub const PURCHASE_RECEIPT_PREFIX: &str = "receipt";
pub const PURCHASE_RECEIPT_SIZE: usize = 8 + std::mem::size_of::<PurchaseReceipt>() + 8;
