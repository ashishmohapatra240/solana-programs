use anchor_lang::prelude::*;

declare_id!("4mpaR6ww1i2r2extRCqLaCtHGKvyes2PvRt6grscdT6n");

#[program]
pub mod todo_crud {
    use super::*;

   pub fn init_user(ctx: Context<InitUser>) -> Result<()> {
    let u = &mut ctx.accounts.user;
    u.authority = ctx.accounts.authority.key();
    u.counter = 0;
    Ok(())
   }

   pub fn create_todo(ctx: Context<CreateTodo>, title: String, note: String) -> Result<()> {
    require!(title.len()<60, TodoError::TitleTooLong);
    require!(note.len()<240, TodoError::NoteTooLong);

    let todo = &mut ctx.accounts.todo;
    let user = &mut ctx.accounts.user;

    todo.authority = ctx.accounts.authority.key();
    todo.idx = user.counter;
    todo.title = title;
    todo.note = note;
    todo.done = false;

    user.counter = user.counter.checked_add(1).ok_or(TodoError::Overflow)?;
    Ok(())
   }

   pub fn update_todo(ctx: Context<UpdateTodo>, new_title: Option<String>, new_note: Option<String>, done:Option<bool>) -> Result<()>{
    let todo = &mut ctx.accounts.todo;

    if let Some(t) = new_title{
        require!(t.len()<60, TodoError::TitleTooLong);

        let new_space = 8 + Todo::base_size() + 4 + t.len() + 4 + todo.note.len();
        let current = todo.to_account_info().data_len();
        if new_space > current {
            todo.to_account_info().resize(new_space)?;
        }
        todo.title =t;
    }
    if let Some(n) = new_note {
        require!(n.len() <= 240, TodoError::NoteTooLong);
        let new_space = 8 + Todo::base_size() + 4 + todo.title.len() + 4 + n.len();
        let current = todo.to_account_info().data_len();
        if new_space > current {
            todo.to_account_info().resize(new_space)?;
        }
        todo.note = n;
    }
    if let Some(d) = done {
        todo.done = d;
    }
    Ok(())
   }

   pub fn delete_todo(ctx: Context<DeleteTodo>) ->Result<()> {
    let todo = &mut ctx.accounts.todo;
    let authority = &mut ctx.accounts.authority;
    let todo_info = todo.to_account_info();
    
    **authority.try_borrow_mut_lamports()? += **todo_info.lamports.borrow();
    **todo_info.lamports.borrow_mut() = 0;
    
    let mut data = todo_info.data.borrow_mut();
    for b in data.iter_mut() { *b = 0}
    Ok(())
   }
}

#[derive(Accounts)]
pub struct InitUser<'info>{
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        init,
        payer = authority,
        seeds = [b"user", authority.key().as_ref()],
        bump,
        space = 8+ User::SIZE
    )]
    pub user: Account<'info, User>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CreateTodo<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        mut,
        seeds = [b"user", authority.key().as_ref()],
        bump
    )]
    pub user: Account<'info, User>,
    #[account(
        init,
        payer = authority,
        seeds = [b"todo", authority.key().as_ref(), &user.counter.to_le_bytes()],
        bump,
        
        space = 8 + Todo::base_size() + 4 + 20 + 4 + 40
    )]
    pub todo: Account<'info, Todo>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateTodo<'info>{
    pub authority: Signer<'info>,
    #[account(
        mut,
        seeds = [b"todo", authority.key().as_ref(), &todo.idx.to_le_bytes()],
        bump
    )]
    pub todo: Account<'info, Todo>,
}


#[derive(Accounts)]
pub struct DeleteTodo<'info>{
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        mut, 
        close = authority,
        seeds=[b"todo", authority.key().as_ref(), &todo.idx.to_le_bytes()],
        bump
    )]
    pub todo: Account<'info, Todo>,
}

#[account]
pub struct User{
    pub authority: Pubkey,
    pub counter: u64,
}
impl User{
    pub const SIZE:usize =32 +8;
}


#[account]
pub struct Todo{
    pub authority: Pubkey,
    pub idx: u64,
    pub title: String,
    pub note: String,
    pub done: bool,
}

impl Todo{
    pub fn base_size() -> usize{
        32 + 8+ 1
    }
}

#[error_code]
pub enum TodoError {
    #[msg("Title too long")]
    TitleTooLong,
    #[msg("Note too long")]
    NoteTooLong,
    #[msg("Overflow")]
    Overflow,
}