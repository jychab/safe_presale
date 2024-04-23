use crate::error::CustomError;
use crate::state::*;
use crate::utils::{Calculator, U128};
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{
    transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked,
};

#[event_cpi]
#[derive(Accounts)]
pub struct WithdrawLpCreatorCtx<'info> {
    #[account(
        mut,
        constraint = pool_lp_token_account.owner == pool.key(),
        constraint = pool_lp_token_account.mint == lp_mint.key(),
    )]
    pub pool_lp_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = lp_mint,
        associated_token::authority = payer,
    )]
    pub pool_authority_lp_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        constraint = lp_mint.key() == pool.lp_mint.unwrap() @CustomError::InvalidRewardMint,
    )]
    pub lp_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        mut, 
        constraint = payer.key() == pool.authority @CustomError::InvalidSigner,
        constraint = pool.vesting_started_at.is_some() @CustomError::PresaleIsStillOngoing,
        seeds = [POOL_PREFIX.as_bytes(), pool.mint.as_ref()],
        bump = pool.bump
    )]
    pub pool: Box<Account<'info, Pool>>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<WithdrawLpCreatorCtx>) -> Result<()> {
    let pool = &mut ctx.accounts.pool;
    let vesting_started_at = pool.vesting_started_at.unwrap();
    let vesting_period = pool.vesting_period;
    let vesting_period_end = vesting_started_at
        .checked_add(vesting_period.into())
        .unwrap();
    let lp_elligible_to_claim = pool.lp_mint_supply_for_creator.unwrap();

    let current_time = Clock::get()?.unix_timestamp;
    let lp_claimable;
    if current_time >= vesting_period_end {
        if lp_elligible_to_claim == pool.lp_mint_claimed_by_creator {
            return Err(error!(CustomError::MaximumAmountClaimed));
        }
        lp_claimable = lp_elligible_to_claim
            .checked_sub(pool.lp_mint_claimed_by_creator)
            .ok_or(CustomError::IntegerOverflow)?;
    } else {
        let last_claimed_at = pool
            .lp_mint_last_claimed_by_creator
            .unwrap_or(vesting_started_at);

        let duration_since_last_claimed = Calculator::to_u64_from_i64(
            current_time
                .checked_sub(last_claimed_at)
                .ok_or(CustomError::IntegerOverflow)?,
        )?;
        lp_claimable = U128::from(duration_since_last_claimed)
            .checked_mul(U128::from(lp_elligible_to_claim))
            .and_then(|result| result.checked_div(vesting_period.try_into().unwrap()))
            .and_then(|result| Some(result.as_u64()))
            .ok_or(CustomError::IntegerOverflow)?;
    }

    //update lp_claimed
    pool.lp_mint_claimed_by_creator = pool
        .lp_mint_claimed_by_creator
        .checked_add(lp_claimable)
        .ok_or(CustomError::IntegerOverflow)?;
    //update last_claimed_at
    pool.lp_mint_last_claimed_by_creator = Some(current_time);

    let pool_seed = &[POOL_PREFIX.as_bytes(), pool.mint.as_ref(), &[pool.bump]];
    let signer = &[&pool_seed[..]];

    //transfer lp
    transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                mint: ctx.accounts.lp_mint.to_account_info(),
                from: ctx.accounts.pool_lp_token_account.to_account_info(),
                to: ctx
                    .accounts
                    .pool_authority_lp_token_account
                    .to_account_info(),
                authority: pool.to_account_info(),
            },
        )
        .with_signer(signer),
        lp_claimable,
        ctx.accounts.lp_mint.decimals,
    )?;

    emit_cpi!(WithdrawLpTokenForCreatorEvent {
        payer: ctx.accounts.payer.key(),
        pool: pool.key(),
        lp_claimed: lp_claimable,
        last_claimed_at: pool.lp_mint_last_claimed_by_creator.unwrap(),
    });

    Ok(())
}
