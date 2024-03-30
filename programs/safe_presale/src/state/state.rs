use anchor_lang::prelude::*;

pub mod wsol {
    use anchor_lang::declare_id;

    declare_id!("So11111111111111111111111111111111111111112");
}
#[account]
pub struct Pool {
    pub bump: u8,
    pub is_closed: bool,
    pub identifier: u64,
    pub authority: Pubkey,
    pub requires_collections: Vec<Pubkey>,
    pub mint: Pubkey,
    pub liquidity_collected: u64,
    pub supply_for_initial_liquidity: u64,
    pub vesting_period: i64,
    pub total_supply: u64,
}

pub const POOL_SIZE: usize = 8 + 1 + 1 + 8 + 32 + 4 + 5 * 32 + 32 + 8 + 8 + 8 + 8;
pub const POOL_PREFIX: &str = "pool";

#[account]
pub struct Identifier {
    pub bump: u8,
    pub count: u64,
}

pub const IDENTIFIER_PREFIX: &str = "identifier";
pub const IDENTIFIER_SIZE: usize = 8 + std::mem::size_of::<Identifier>() + 8;

pub const MINT_TOKEN_PREFIX: &str = "mint";

#[account]
pub struct PurchaseReceipt {
    pub bump: u8,
    pub pool: Pubkey,
    pub amount: u64,
    pub original_mint: Pubkey,
    pub vesting_started_at: i64,
    pub last_claimed_at: Option<i64>,
}

pub const PURCHASE_RECEIPT_PREFIX: &str = "receipt";
pub const PURCHASE_RECEIPT_SIZE: usize = 8 + std::mem::size_of::<PurchaseReceipt>() + 8;
