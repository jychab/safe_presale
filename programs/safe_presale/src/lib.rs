use anchor_lang::prelude::*;

declare_id!("6DvsdPa3nXFjFG8ENuABoWeRjQaHUks2pKKqGD9ihekw");

pub mod error;
pub mod instructions;
pub mod state;

pub use instructions::*;

#[program]
pub mod safe_presale {

    use super::*;

    pub fn buy_presale(ctx: Context<BuyPresaleCtx>, amount: u64) -> Result<()> {
        instructions::buy_presale::handler(ctx, amount)
    }
    pub fn claim_rewards(ctx: Context<ClaimRewardsCtx>) -> Result<()> {
        instructions::claim_rewards::handler(ctx)
    }
    pub fn init_identifier(ctx: Context<InitIdentifierCtx>) -> Result<()> {
        instructions::init_identifier::handler(ctx)
    }
    pub fn init_pool(ctx: Context<InitPoolCtx>, args: InitPoolArgs) -> Result<()> {
        instructions::init_pool::handler(ctx, args)
    }
    pub fn launch_token_clmm<'a, 'b, 'c: 'info, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, LaunchTokenClmmCtx<'info>>,
        liquidity: u128,
        amount_0_max: u64,
        amount_1_max: u64,
        tick_lower_index: i32,
        tick_upper_index: i32,
        tick_array_lower_start_index: i32,
        tick_array_upper_start_index: i32,
    ) -> Result<()> {
        instructions::launch_token_clmm::handler(
            ctx,
            liquidity,
            amount_0_max,
            amount_1_max,
            tick_lower_index,
            tick_upper_index,
            tick_array_lower_start_index,
            tick_array_upper_start_index,
        )
    }
}
