use anchor_lang::prelude::*;

declare_id!("9i5B2qB2X1UwRwT6wrFACz7QVMfbHgFPcgQx38KU2hk9");

#[program]
pub mod pda_counter {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, seed: String) -> Result<()> {
        let c = &mut ctx.accounts.counter;
        c.authority = ctx.accounts.authority.key();
        c.value = 0;
        c.seed = seed;
        Ok(())
    }

    pub fn increment(ctx: Context<Update>) -> Result<()> {
        let c = &mut ctx.accounts.counter;
        require_keys_eq!(
            c.authority,
            ctx.accounts.authority.key(),
            CounterError::Unauthorized
        );
        c.value = c.value.checked_add(1).ok_or(CounterError::Overflow)?;
        Ok(())
    }

    pub fn set(ctx: Context<Update>, new_value: u64) -> Result<()> {
        let c = &mut ctx.accounts.counter;
        require_keys_eq!(
            c.authority,
            ctx.accounts.authority.key(),
            CounterError::Unauthorized
        );
        c.value = new_value;
        Ok(())
    }

    pub fn reset(ctx: Context<Update>) -> Result<()> {
        let c = &mut ctx.accounts.counter;
        require_keys_eq!(
            c.authority,
            ctx.accounts.authority.key(),
            CounterError::Unauthorized
        );
        c.value = 0;
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(seed:String)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        init,
        payer = authority,
        seeds = [b"counter", authority.key().as_ref(), seed.as_bytes()],
        bump,
        space = 8 + Counter::MAX_SIZE
    )]
    pub counter: Account<'info, Counter>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Update<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        mut,
        seeds = [b"counter", counter.authority.as_ref(), counter.seed.as_bytes()],
        bump
    )]
    pub counter: Account<'info, Counter>,
}

#[account]
pub struct Counter {
    pub authority: Pubkey,
    pub value: u64,
    pub seed: String,
}
impl Counter {
    pub const MAX_SIZE: usize = 32 + 8 + 4 + 32;
}

#[error_code]
pub enum CounterError {
    #[msg("Unauthorized")]
    Unauthorized,
    #[msg("Overflow")]
    Overflow,
}
