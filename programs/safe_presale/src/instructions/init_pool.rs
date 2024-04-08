use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token_interface::{Mint, MintTo, mint_to, TokenAccount, TokenInterface}};
use mpl_token_metadata::{instructions::CreateMetadataAccountV3CpiBuilder, types::DataV2};
use crate::{error::CustomError, state::{Identifier, InitializedPoolEvent, Pool, MINT_PREFIX, POOL_PREFIX, POOL_SIZE}};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct InitPoolArgs {
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub decimals: u8,
    pub presale_target: u64,
    pub max_presale_time: u32,
    pub vesting_period: u32,
    pub vested_supply: u64,
    pub total_supply: u64,
    pub creator_fee_basis_points: u16,
}

#[event_cpi]
#[derive(Accounts)]
#[instruction(args:InitPoolArgs)]
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
        seeds = [MINT_PREFIX.as_bytes(), identifier.count.to_le_bytes().as_ref()],
        bump,
        mint::decimals = args.decimals,
        mint::authority = pool,
    )]
    pub reward_mint: Box<InterfaceAccount<'info, Mint>>,

    /// CHECK: Checked by cpi
    #[account(mut)]
    pub reward_mint_metadata: UncheckedAccount<'info>,

    #[account( 
        init_if_needed,
        payer = payer,  
        associated_token::mint = reward_mint,
        associated_token::authority = pool,
    )]
    pub pool_reward_mint_ata: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(mut)]
    pub identifier: Account<'info, Identifier>,

    #[account(mut)]
    pub payer: Signer<'info>,

    /// CHECK: This is not dangerous because we don't read or write from this account
    pub system_program: Program<'info, System>,

    pub token_program: Interface<'info, TokenInterface>,

    pub associated_token_program: Program<'info, AssociatedToken>,

    /// CHECK: Checked by cpi
    #[account(address = mpl_token_metadata::ID)]
    pub mpl_token_program: AccountInfo<'info>,
}

pub fn handler(ctx: Context<InitPoolCtx>, args: InitPoolArgs) -> Result<()> {
    if args.vested_supply > args.total_supply {
        return Err(error!(CustomError::VestingSupplyLargerThanTotalSupply))
    }
    if args.creator_fee_basis_points > 10000 {
        return Err(error!(CustomError::CreatorBasisPointsExceedMaximumAmount))
    }
    let pool = &mut ctx.accounts.pool;
    let identifier = &mut ctx.accounts.identifier;
    let current_time = Clock::get()?.unix_timestamp;
    pool.launched = false;
    pool.bump = ctx.bumps.pool;
    pool.identifier = identifier.count;
    pool.mint = ctx.accounts.reward_mint.key();
    pool.authority = ctx.accounts.payer.key();
    pool.liquidity_collected = 0;
    pool.total_supply = args.total_supply.checked_mul(10u64.checked_pow(args.decimals.into()).unwrap()).unwrap();
    pool.vested_supply = args.vested_supply.checked_mul(10u64.checked_pow(args.decimals.into()).unwrap()).unwrap();
    pool.presale_time_limit = current_time.checked_add(args.max_presale_time.try_into().unwrap()).ok_or(CustomError::IntegerOverflow)?;
    pool.vesting_period = args.vesting_period;
    pool.creator_fee_basis_points = args.creator_fee_basis_points;
    pool.presale_target = args.presale_target;


    let pool_identifier = pool.identifier.to_le_bytes();
    let seeds = &[
        POOL_PREFIX.as_bytes(),
        pool_identifier.as_ref(),
        &[pool.bump],
    ];
    let signer = &[&seeds[..]];

    //mint remaining token to pool
    let cpi_accounts = MintTo {
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
    let amount_to_mint = pool.total_supply.checked_sub(pool.vested_supply).ok_or(CustomError::IntegerOverflow)?;
    mint_to(
        cpi_context,
        amount_to_mint
    )?;

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

    // Emit the Initialzed pool event
    emit_cpi!(InitializedPoolEvent {
        authority: pool.authority,
        pool: pool.key(),
        mint: pool.mint,
        decimal: args.decimals,
        presale_target: pool.presale_target,
        presale_time_limit: pool.presale_time_limit,
        creator_fee_basis_points: pool.creator_fee_basis_points,
        vested_supply: pool.vested_supply,
        total_supply: pool.total_supply,
        vesting_period: pool.vesting_period,
    });

    Ok(())
}
