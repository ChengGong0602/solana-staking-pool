use anchor_lang::prelude::*;
// Import this package to manage spl-tokens
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};
use std::ops::Deref;


declare_id!("HWJiQ61uFie8iQqSH3P76ukWYZEvwLJj29RwQZY48C33");

// TOKEN NAME "SEEDED"
// Token decimal
const DECIMALS: u8 = 9;
// Seconds for a day
const DAY_IN_SECONDS: i64 = 86400;
// Reward Rate => 0.05%
// so need to divide with 10000
const DAILY_REWARD_RATE: u64 = 5;


#[program]
pub mod staking_pool {
    use super::*;
    pub fn initialize(
        ctx: Context<Initialize>,
        program_title: String,
        bumps: PoolBumps
    ) -> ProgramResult {
        msg!("INITIALIZE POOL");
        
        let store_account = &mut ctx.accounts.store_account;

        let name_bytes = program_title.as_bytes();
        let mut name_data = [b' '; 10];
        name_data[..name_bytes.len()].copy_from_slice(name_bytes);

        store_account.program_title = name_data;
        store_account.bumps = bumps;

        store_account.store_authority = ctx.accounts.store_authority.key();
        store_account.seeded_mint = ctx.accounts.seeded_mint.key();
        store_account.pool_seeded = ctx.accounts.pool_seeded.key();

        Ok(())
    }

    // #[access_control(unrestricted_phase(&ctx.accounts.ido_account))]
    pub fn init_stake_account(ctx: Context<InitAccount>, bump: u8, seed0: String, seed1: String) -> ProgramResult {
        let mut owned_string: String = seed0.to_owned();
        let another_owned_string: String = seed1.to_owned();
        owned_string.push_str(&another_owned_string);
        
        msg!("INIT STAKE ACCOUNT {:?}, {:?}", owned_string, ctx.accounts.store_authority.key().to_string());
        if owned_string != ctx.accounts.store_authority.key().to_string() {
            return Err(ErrorCode::FailedInit.into())
        }

        let stake_account = &mut ctx.accounts.stake_account;
        stake_account.owner = ctx.accounts.store_authority.key();
        stake_account.bump = bump;
        Ok(())
    }

    // stake SEEDED token into player
    pub fn enter_staking(
        ctx: Context<EnterStaking>,
        amount: u64
    ) -> ProgramResult {
        msg!("Enter staking!!!");
        if amount < 1 {
            return Err(ErrorCode::ZeroSeeded.into())
        }

        if ctx.accounts.user_seeded.amount < amount {
            return Err(ErrorCode::LowSeeded.into())
        }

        // Transfer user's SEEDED to pool SEEDED account.
        let cpi_accounts = Transfer {
            // storer address (user address)
            from: ctx.accounts.user_seeded.to_account_info(),
            to: ctx.accounts.pool_seeded.to_account_info(),
            authority: ctx.accounts.user_authority.to_account_info(),
        };

        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, amount)?;

        let clock = Clock::get()?; // Returns real-world time in second uint
        let stake_account = &mut ctx.accounts.stake_account;
        let earned_amount = stake_account.earned_amount;
        let staked_amount = stake_account.staked_amount;

        let dur_seconds = clock.unix_timestamp - stake_account.last_stake_ts;
        let staked_days = dur_seconds / DAY_IN_SECONDS;
        // Need to divide with 10000 cuz rate is 0.05%;
        let added_reward_amount = staked_amount * staked_days as u64 * DAILY_REWARD_RATE / 10000;

        stake_account.staked_amount = staked_amount + amount;
        stake_account.earned_amount = earned_amount + added_reward_amount;
        stake_account.last_stake_ts = clock.unix_timestamp;

        Ok(())
    }

    pub fn start_unstaking(
        ctx: Context<UnStaking>,
        amount: u64
    ) -> ProgramResult {
        let stake_account = &mut ctx.accounts.stake_account;

        if stake_account.staked_amount < 1  {
            return Err(ErrorCode::NoStaked.into())
        }

        if stake_account.staked_amount < amount  {
            return Err(ErrorCode::LowWithdraw.into())
        }

        let clock = Clock::get()?; // Returns real-world time in second uint
        let stake_account = &mut ctx.accounts.stake_account;
        let earned_amount = stake_account.earned_amount;
        let staked_amount = stake_account.staked_amount;

        let dur_seconds = clock.unix_timestamp - stake_account.last_stake_ts;
        let staked_days = dur_seconds / DAY_IN_SECONDS;
        // Need to divide with 10000 cuz rate is 0.05%;
        let added_reward_amount = staked_amount * staked_days as u64 * DAILY_REWARD_RATE / 10000;

        let withdrawal_amount = amount + earned_amount + added_reward_amount;

        if ctx.accounts.pool_seeded.amount < withdrawal_amount  {
            return Err(ErrorCode::NoEnoughPool.into())
        }

        stake_account.staked_amount = staked_amount - amount;
        stake_account.earned_amount = 0;
        stake_account.last_stake_ts = clock.unix_timestamp;

        // Transfer seeded from pool account to the user's account.
        let program_title = ctx.accounts.store_account.program_title.as_ref();
        let seeds = &[
            program_title.trim_ascii_whitespace(),
            &[ctx.accounts.store_account.bumps.store_account],
        ];
        let signer = &[&seeds[..]];

        let cpi_accounts = Transfer {
            from: ctx.accounts.pool_seeded.to_account_info(),
            to: ctx.accounts.user_seeded.to_account_info(),
            authority: ctx.accounts.store_account.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);

        token::transfer(cpi_ctx, withdrawal_amount)?;

        Ok(())
    }

    pub fn harvest(
        ctx: Context<Harvest>
    ) -> ProgramResult {
        let stake_account = &mut ctx.accounts.stake_account;

        if stake_account.staked_amount < 1  {
            return Err(ErrorCode::NoStaked.into())
        }

        let clock = Clock::get()?; // Returns real-world time in second uint
        let stake_account = &mut ctx.accounts.stake_account;
        let earned_amount = stake_account.earned_amount;
        let staked_amount = stake_account.staked_amount;

        let dur_seconds = clock.unix_timestamp - stake_account.last_stake_ts;
        let staked_days = dur_seconds / DAY_IN_SECONDS;
        // Need to divide with 10000 cuz rate is 0.05%;
        let added_reward_amount = staked_amount * staked_days as u64 * DAILY_REWARD_RATE / 10000;

        let harvest_amount = earned_amount + added_reward_amount;

        if harvest_amount < 1  {
            return Err(ErrorCode::NoEnoughHarvest.into())
        }

        if ctx.accounts.pool_seeded.amount < harvest_amount  {
            return Err(ErrorCode::NoEnoughPool.into())
        }

        stake_account.earned_amount = 0;
        stake_account.last_stake_ts = clock.unix_timestamp;

        // Transfer seeded from pool account to the user's account.
        let program_title = ctx.accounts.store_account.program_title.as_ref();
        let seeds = &[
            program_title.trim_ascii_whitespace(),
            &[ctx.accounts.store_account.bumps.store_account],
        ];
        let signer = &[&seeds[..]];

        let cpi_accounts = Transfer {
            from: ctx.accounts.pool_seeded.to_account_info(),
            to: ctx.accounts.user_seeded.to_account_info(),
            authority: ctx.accounts.store_account.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);

        token::transfer(cpi_ctx, harvest_amount)?;
        Ok(())
    }

}

#[derive(Accounts)]
#[instruction(program_title: String, bumps: PoolBumps)]
pub struct Initialize <'info> {
    // Store Accounts
    #[account(init,
        seeds = [program_title.as_bytes()],
        bump = bumps.store_account,
        payer = store_authority
    )]
    pub store_account: Account<'info, StoreAccount>,
    // Contract Authority accounts
    #[account(mut)]
    pub store_authority: Signer<'info>,
    // Staking token
    #[account(constraint = seeded_mint.decimals == DECIMALS)]
    pub seeded_mint: Account<'info, Mint>,

    #[account(
        init,
        token::mint = seeded_mint,
        token::authority = store_account,
        seeds = [program_title.as_bytes(), b"pool_seeded".as_ref()],
        bump = bumps.pool_seeded,
        payer = store_authority
    )]
    pub pool_seeded: Account<'info, TokenAccount>,

    // Programs and Sysvars
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}


#[derive(Accounts)]
#[instruction(bump: u8, seed0: String, seed1: String)]
pub struct InitAccount<'info> {
    // Store Accounts
    #[account(init,
        seeds = [store_account.program_title.as_ref(), seed0.as_ref(), seed1.as_ref()],
        bump = bump,
        payer = store_authority
    )]
    pub stake_account: Account<'info, StakeInfoAccount>,
    pub store_account: Account<'info, StoreAccount>,
    // Contract Authority accounts
    #[account(mut)]
    pub store_authority: Signer<'info>,
    // Programs and Sysvars
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct EnterStaking<'info> {
    // User Accounts
    #[account(mut)]
    pub user_authority: Signer<'info>,
    // TODO replace these with the ATA constraints when possible
    #[account(
        mut,
        constraint = user_seeded.owner == user_authority.key(),
        constraint = user_seeded.mint == seeded_mint.key()
    )]
    pub user_seeded: Account<'info, TokenAccount>,
    #[account(mut)]
    pub seeded_mint: Account<'info, Mint>,
    
    #[account(mut,
        constraint = stake_account.owner == user_authority.key()
    )]
    pub stake_account: Account<'info, StakeInfoAccount>,

    #[account(mut, seeds = [store_account.program_title.as_ref().trim_ascii_whitespace()],
        bump = store_account.bumps.store_account,
        has_one = seeded_mint)]
    pub store_account: Box<Account<'info, StoreAccount>>,
    
    #[account(mut,
        seeds = [store_account.program_title.as_ref().trim_ascii_whitespace(), b"pool_seeded".as_ref()],
        bump = store_account.bumps.pool_seeded)]
    pub pool_seeded: Account<'info, TokenAccount>,
    // Programs and Sysvars
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct UnStaking<'info> {
    // User Accounts
    #[account(mut)]
    pub user_authority: Signer<'info>,
    // TODO replace these with the ATA constraints when possible
    #[account(mut,
        constraint = user_seeded.owner == user_authority.key(),
        constraint = user_seeded.mint == seeded_mint.key())]
    pub user_seeded: Account<'info, TokenAccount>,
    pub seeded_mint: Account<'info, Mint>,

    // store Accounts
    #[account(seeds = [store_account.program_title.as_ref().trim_ascii_whitespace()],
        bump = store_account.bumps.store_account,
        has_one = seeded_mint)]
    pub store_account: Box<Account<'info, StoreAccount>>,
    // Stake Account
    #[account(mut,
        constraint = stake_account.owner == user_authority.key())]
    pub stake_account: Account<'info, StakeInfoAccount>,

    #[account(mut,
        seeds = [store_account.program_title.as_ref().trim_ascii_whitespace(), b"pool_seeded".as_ref()],
        bump = store_account.bumps.pool_seeded)]
    pub pool_seeded: Account<'info, TokenAccount>,
    // Programs and Sysvars
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Harvest<'info> {
    // User Accounts
    #[account(mut)]
    pub user_authority: Signer<'info>,
    // TODO replace these with the ATA constraints when possible
    #[account(mut,
        constraint = user_seeded.owner == user_authority.key(),
        constraint = user_seeded.mint == seeded_mint.key())]
    pub user_seeded: Account<'info, TokenAccount>,
    pub seeded_mint: Account<'info, Mint>,

    // store Accounts
    #[account(seeds = [store_account.program_title.as_ref().trim_ascii_whitespace()],
        bump = store_account.bumps.store_account,
        has_one = seeded_mint)]
    pub store_account: Box<Account<'info, StoreAccount>>,
    // Stake Account
    #[account(mut)]
    pub stake_account: Account<'info, StakeInfoAccount>,
    #[account(mut,
        seeds = [store_account.program_title.as_ref().trim_ascii_whitespace(), b"pool_seeded".as_ref()],
        bump = store_account.bumps.pool_seeded)]
    pub pool_seeded: Account<'info, TokenAccount>,
    // Programs and Sysvars
    pub token_program: Program<'info, Token>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Default, Clone)]
pub struct PoolBumps {
    pub store_account: u8,
    pub pool_seeded: u8,
}

#[error]
pub enum ErrorCode {
    #[msg("Zero amount")]
    ZeroSeeded,
    #[msg("NO WITHDRAWABLE SEEDED")]
    EmptySeeded,
    #[msg("POOL IS NOT ENOUGH TO WITHDRAWAL OR HARVEST")]
    NoEnoughPool,
    #[msg("POOL IS NOT ENOUGH TO HARVEST")]
    NoEnoughHarvest,
    #[msg("Insufficient withdrawal tokens")]
    LowWithdraw,
    #[msg("NO STAKED")]
    NoStaked,
    #[msg("Insufficient SEEDED")]
    LowSeeded,
    #[msg("Initialize Stake Account Failed")]
    FailedInit,
    #[msg("Insufficient redeemable tokens")]
    LowRedeemable,
    #[msg("SEEDED total and redeemable total don't match")]
    SeededNotEqRedeem,
}

#[account]
#[derive(Default)]
pub struct StoreAccount {
    pub program_title: [u8; 10], // Setting an arbitrary max of ten characters in the ido name.
    pub bumps: PoolBumps,
    pub store_authority: Pubkey,

    pub seeded_mint: Pubkey,
    pub pool_seeded: Pubkey
}

#[account]
#[derive(Default)]
pub struct StakeInfoAccount {
    pub owner: Pubkey,
    pub bump: u8,
    pub staked_amount: u64,
    pub earned_amount: u64,
    pub last_stake_ts: i64
}

/// Trait to allow trimming ascii whitespace from a &[u8].
pub trait TrimAsciiWhitespace {
    /// Trim ascii whitespace (based on `is_ascii_whitespace()`) from the
    /// start and end of a slice.
    fn trim_ascii_whitespace(&self) -> &[u8];
}

impl<T: Deref<Target = [u8]>> TrimAsciiWhitespace for T {
       
    fn trim_ascii_whitespace(&self) -> &[u8] {
        let from = match self.iter().position(|x| !x.is_ascii_whitespace()) {
            Some(i) => i,
            None => return &self[0..0],
        };
        let to = self.iter().rposition(|x| !x.is_ascii_whitespace()).unwrap();
        &self[from..=to]
    }
}
