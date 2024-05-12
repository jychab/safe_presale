use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token_2022::spl_token_2022::instruction::AuthorityType, token_interface::{mint_to, set_authority, Mint, MintTo, SetAuthority, TokenAccount, TokenInterface}};
use mpl_token_metadata::{instructions::CreateMetadataAccountV3CpiBuilder, types::DataV2};
use crate::{error::CustomError, state::{InitializedPoolEvent, Pool, MINT_PREFIX, POOL_PREFIX, POOL_SIZE}, utils::U128};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct InitPoolArgs {
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub decimals: u8,
    pub presale_target: u64, 
    pub presale_duration: u32,
    pub vesting_period: u32,
    pub max_amount_per_purchase: Option<u64>,
    pub liquidity_pool_supply: u64,
    pub initial_supply: u64,
    pub creator_fee_basis_points: u16,
    pub delegate: Option<Pubkey>,
    pub random_key: u64,
    pub requires_collection: bool,
    pub quote_mint: Pubkey,
}

#[event_cpi]
#[derive(Accounts)]
#[instruction(args:InitPoolArgs)]
pub struct InitPoolCtx<'info> {
    #[account(
        init,
        payer = payer,
        seeds = [MINT_PREFIX.as_bytes(), args.random_key.to_le_bytes().as_ref()],
        bump,
        mint::decimals = args.decimals,
        mint::authority = pool,
    )]
    pub reward_mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        init,
        payer=payer,
        space = POOL_SIZE,
        seeds = [POOL_PREFIX.as_bytes(), reward_mint.key().as_ref()],
        bump,
    )]
    pub pool: Box<Account<'info, Pool>>,

    /// CHECK: Checked by cpi
    #[account(mut)]
    pub reward_mint_metadata: UncheckedAccount<'info>,

    #[account( 
        init_if_needed,
        payer = payer,  
        associated_token::mint = reward_mint,
        associated_token::authority = pool,
    )]
    pub pool_reward_mint_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

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
    require!(args.creator_fee_basis_points <= 10000, CustomError::CreatorBasisPointsExceedMaximumAmount);

    let pool = &mut ctx.accounts.pool;
    let current_time = Clock::get()?.unix_timestamp;
    pool.bump = ctx.bumps.pool;
    pool.mint = ctx.accounts.reward_mint.key();
    pool.authority = ctx.accounts.payer.key();
    pool.liquidity_collected = 0;
    pool.liquidity_pool_supply = args.liquidity_pool_supply.checked_mul(10u64.checked_pow(args.decimals.into()).unwrap()).unwrap();
    pool.initial_supply = args.initial_supply.checked_mul(10u64.checked_pow(args.decimals.into()).unwrap()).unwrap();
    pool.presale_time_limit = current_time.checked_add(args.presale_duration.try_into().unwrap()).ok_or(CustomError::IntegerOverflow)?;
    pool.vesting_period = args.vesting_period;
    pool.creator_fee_basis_points = args.creator_fee_basis_points;
    pool.presale_target = args.presale_target;
    pool.delegate = args.delegate;
    pool.max_amount_per_purchase = args.max_amount_per_purchase;
    pool.requires_collection = args.requires_collection;
    pool.quote_mint = args.quote_mint;
    pool.initial_supply_for_creator = U128::from(pool.initial_supply)
    .checked_mul(args.creator_fee_basis_points.try_into().unwrap())
    .and_then(|result| result.checked_div(U128::from(10000)))
    .and_then(|result| Some(result.as_u64()))
    .ok_or(CustomError::IntegerOverflow)?;


    let seeds = &[
        POOL_PREFIX.as_bytes(),
        pool.mint.as_ref(),
        &[pool.bump],
    ];
    let signer = &[&seeds[..]];

    //mint all supply to pool then revoke mint authority for token
    mint_to(
        CpiContext::new(ctx.accounts.token_program.to_account_info(), MintTo {
            mint: ctx.accounts.reward_mint.to_account_info(),
            to: ctx
                .accounts
                .pool_reward_mint_token_account
                .to_account_info(),
            authority: pool.to_account_info(),
        })
        .with_signer(signer),
        pool.liquidity_pool_supply.checked_add(pool.initial_supply).ok_or(CustomError::IntegerOverflow)?
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

    //set mint authority to none
    set_authority(CpiContext::new(ctx.accounts.token_program.to_account_info(), SetAuthority{
        current_authority: pool.to_account_info(),
        account_or_mint: ctx.accounts.reward_mint.to_account_info(),
    }).with_signer(signer), 
    AuthorityType::MintTokens, 
    None
    )?;

    // Emit the Initialzed pool event
    emit_cpi!(InitializedPoolEvent {
        delegate: pool.delegate,
        authority: pool.authority,
        pool: pool.key(),
        mint: pool.mint,
        decimal: args.decimals,
        presale_target: pool.presale_target,
        presale_time_limit: pool.presale_time_limit,
        creator_fee_basis_points: pool.creator_fee_basis_points,
        liquidity_pool_supply: pool.liquidity_pool_supply,
        initial_supply: pool.initial_supply,
        initial_supply_for_creator: pool.initial_supply_for_creator,
        vesting_period: pool.vesting_period,
        max_amount_per_purchase: pool.max_amount_per_purchase,
        requires_collection: args.requires_collection,
        quote_mint: pool.quote_mint,
    });

    Ok(())
}
