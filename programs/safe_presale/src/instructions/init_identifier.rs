use crate::state::*;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct InitIdentifierCtx<'info> {
    #[account(
        init,
        payer = payer,
        space = IDENTIFIER_SIZE,
        seeds = [IDENTIFIER_PREFIX.as_bytes()],
        bump
    )]
    pub identifier: Account<'info, Identifier>,

    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<InitIdentifierCtx>) -> Result<()> {
    let identifier = &mut ctx.accounts.identifier;
    identifier.bump = ctx.bumps.identifier;
    identifier.count = 1;

    Ok(())
}
