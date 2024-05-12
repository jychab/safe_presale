use crate::error::CustomError;
use crate::state::*;
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{
    transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked,
};

#[event_cpi]
#[derive(Accounts)]
pub struct ClaimRewardCreatorCtx<'info> {
    #[account(
        mut,
        constraint = pool_reward_token_account.owner == pool.key(),
        constraint = pool_reward_token_account.mint == pool.mint,
    )]
    pub pool_reward_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = reward_mint,
        associated_token::authority = payer,
    )]
    pub pool_authority_reward_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        constraint = reward_mint.key() == pool.mint @CustomError::InvalidRewardMint,
    )]
    pub reward_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        constraint = payer.key() == pool.authority @CustomError::InvalidSigner,
        constraint = pool.vesting_started_at.is_some() @CustomError::PresaleIsStillOngoing,
    )]
    pub pool: Box<Account<'info, Pool>>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<ClaimRewardCreatorCtx>) -> Result<()> {
    let pool = &ctx.accounts.pool;
    let pool_seed = &[POOL_PREFIX.as_bytes(), pool.mint.as_ref(), &[pool.bump]];
    let signer = &[&pool_seed[..]];

    //transfer mint
    transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                mint: ctx.accounts.reward_mint.to_account_info(),
                from: ctx.accounts.pool_reward_token_account.to_account_info(),
                to: ctx
                    .accounts
                    .pool_authority_reward_token_account
                    .to_account_info(),
                authority: pool.to_account_info(),
            },
        )
        .with_signer(signer),
        pool.initial_supply_for_creator,
        ctx.accounts.reward_mint.decimals,
    )?;

    emit_cpi!(ClaimRewardForCreatorEvent {
        payer: ctx.accounts.payer.key(),
        pool: pool.key(),
        mint_elligible: pool.initial_supply_for_creator,
    });

    Ok(())
}
