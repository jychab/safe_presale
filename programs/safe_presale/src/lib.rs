use anchor_lang::prelude::*;

declare_id!("9jqqZaNVmt9w4umoa7AajntuQiQPhfD5SMKMEpyk3R3Z");

pub mod error;
pub mod instructions;
pub mod state;
pub mod utils;

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
    pub fn withdraw_lp_token<'info>(ctx: Context<WithdrawPoolLpToken<'info>>) -> Result<()> {
        instructions::withdraw_lp_token::handler(ctx)
    }
    pub fn launch_token_amm<'a, 'b, 'c: 'info, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, LaunchTokenAmmCtx<'info>>,
        nonce: u8,
        open_time: u64,
    ) -> Result<()> {
        instructions::launch_token_amm::handler(ctx, nonce, open_time)
    }
}
