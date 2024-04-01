use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token::{self, Mint, Token, TokenAccount}};
use mpl_token_metadata::{instructions::CreateMetadataAccountV3CpiBuilder, types::DataV2};

use crate::{error::CustomError, state::{Identifier, Pool, POOL_PREFIX, POOL_SIZE}};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct InitPoolArgs {
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub decimals: u8,
    pub vesting_period: u64,
    pub vested_supply: u64,
    pub total_supply: u64,
    pub creator_fee_basis_points: u16, 
    pub requires_collections: Vec<Pubkey>,
}

#[derive(Accounts)]
#[instruction(params:InitPoolArgs)]
pub struct InitPoolCtx<'info> {
    #[account(
        init,
        payer=payer,
        space = POOL_SIZE,
        seeds = [POOL_PREFIX.as_bytes(), identifier.count.to_le_bytes().as_ref()],
        bump,
    )]
    pub pool: Box<Account<'info, Pool>>,

    #[account(
        init,
        payer = payer,
        mint::decimals = params.decimals,
        mint::authority = pool,
    )]
    pub reward_mint: Box<Account<'info, Mint>>,

    /// CHECK: Checked by cpi
    #[account(mut)]
    pub reward_mint_metadata: UncheckedAccount<'info>,

    #[account( 
        init_if_needed,
        payer = payer,
        associated_token::mint = reward_mint,
        associated_token::authority = pool,
    )]
    pub pool_reward_mint_ata: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub identifier: Account<'info, Identifier>,

    #[account(mut)]
    pub payer: Signer<'info>,

    /// CHECK: This is not dangerous because we don't read or write from this account
    pub system_program: Program<'info, System>,

    pub token_program: Program<'info, Token>,

    pub associated_token_program: Program<'info, AssociatedToken>,

    /// CHECK: Checked by cpi
    #[account(address = mpl_token_metadata::ID)]
    pub mpl_token_program: AccountInfo<'info>,
}

pub fn handler(ctx: Context<InitPoolCtx>, args: InitPoolArgs) -> Result<()> {
    msg!("Intializing pool");
    let pool = &mut ctx.accounts.pool;
    let identifier = &mut ctx.accounts.identifier;
    pool.allow_purchase = true;
    pool.bump = ctx.bumps.pool;
    pool.identifier = identifier.count;
    pool.requires_collections = args.requires_collections;
    pool.mint = ctx.accounts.reward_mint.key();
    pool.authority = ctx.accounts.payer.key();
    pool.liquidity_collected = 0;

    
    if args.vested_supply > args.total_supply {
        return Err(error!(CustomError::VestingSupplyLargerThanTotalSupply))
    }
    if args.creator_fee_basis_points > 10000 {
        return Err(error!(CustomError::CreatorBasisPointsExceedMaximumAmount))
    }

    pool.total_supply = args.total_supply;
    pool.vested_supply = args.vested_supply;
    pool.vesting_period = args.vesting_period;
    pool.creator_fee_basis_points = args.creator_fee_basis_points;

    msg!("Creating Pool seeds");
    let pool_identifier = pool.identifier.to_le_bytes();
    let seeds = &[
        POOL_PREFIX.as_bytes(),
        pool_identifier.as_ref(),
        &[pool.bump],
    ];
    let signer = &[&seeds[..]];

    msg!("Minting Remaining Token To Pool");
    //mint remaining token to pool
    let cpi_accounts = token::MintTo {
        mint: ctx.accounts.reward_mint.to_account_info(),
        to: ctx
            .accounts
            .pool_reward_mint_ata
            .to_account_info(),
        authority: pool.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_context = CpiContext::new(cpi_program, cpi_accounts)
        .with_signer(signer);
    token::mint_to(
        cpi_context,
        pool.total_supply - pool.vested_supply
    )?;

    msg!("Creating metadata");
    CreateMetadataAccountV3CpiBuilder::new(&ctx.accounts.mpl_token_program.to_account_info())
    .system_program(&ctx.accounts.system_program.to_account_info())
    .mint(&ctx.accounts.reward_mint.to_account_info())
    .mint_authority(&pool.to_account_info())
    .metadata(&ctx.accounts.reward_mint_metadata.to_account_info())
    .is_mutable(false)
    .payer(&ctx.accounts.payer)
    .update_authority(&ctx.accounts.payer,true)
    .data(DataV2{
        name: args.name,
        symbol: args.symbol,
        uri: args.uri,
        seller_fee_basis_points: 0,
        creators: None,
        collection: None,
        uses: None,
    }).invoke_signed(signer)?;

    identifier.count += 1;
    Ok(())
}
