use anchor_lang::prelude::*;

declare_id!("memep6GYetMx84qtBgB9p1rncn81HMmZZa1UoxauYGt");

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
    pub fn withdraw_lp_tokens(ctx: Context<WithdrawLpCtx>) -> Result<()> {
        instructions::withdraw_lp_tokens::handler(ctx)
    }
    pub fn withdraw_lp_tokens_for_creators(ctx: Context<WithdrawLpCreatorCtx>) -> Result<()> {
        instructions::withdraw_lp_tokens_for_creator::handler(ctx)
    }
    pub fn claim_reward_token(ctx: Context<ClaimRewardCtx>) -> Result<()> {
        instructions::claim_reward_token::handler(ctx)
    }
    pub fn claim_reward_token_for_creators(ctx: Context<ClaimRewardCreatorCtx>) -> Result<()> {
        instructions::claim_reward_token_for_creator::handler(ctx)
    }
    pub fn init_pool(ctx: Context<InitPoolCtx>, args: InitPoolArgs) -> Result<()> {
        instructions::init_pool::handler(ctx, args)
    }
    pub fn create_purchase_authorisation(
        ctx: Context<CreatePurchaseAuthorizationCtx>,
        collection_mint: Pubkey,
    ) -> Result<()> {
        instructions::create_purchase_authorisation::handler(ctx, collection_mint)
    }
    pub fn withdraw<'info>(ctx: Context<Withdraw<'info>>) -> Result<()> {
        instructions::withdraw::handler(ctx)
    }
    pub fn check_claim_ellgibility(ctx: Context<CheckClaimCtx>) -> Result<()> {
        instructions::check_claim_elligibility::handler(ctx)
    }
    pub fn launch_token_amm<'a, 'b, 'c: 'info, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, LaunchTokenAmmCtx<'info>>,
        nonce: u8,
        open_time: u64,
    ) -> Result<()> {
        instructions::launch_token_amm::handler(ctx, nonce, open_time)
    }
}
