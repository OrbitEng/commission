use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

pub mod accessors;
pub mod structs;

pub use accessors::*;
pub use structs::*;

#[program]
pub mod orbit_commission {
    use super::*;

    ////////////////////////////////
    /// COMMISSION RELATED
    
    // pub fn create_commission_account(ctx: Context<CreateComishAccount>) -> Result<()>{
    //     create_commission_account_handler(ctx)
    // }
    // pub fn commit_preview(ctx: Context<CommitPreview>, link: [u8; 64]) -> Result<()>{
    //     commit_preview_handler(ctx, link)
    // }
    // pub fn propose_rate(ctx: Context<UpdateRate>, new_rate: u8) -> Result<()>{
    //     propose_rate_handler(ctx, new_rate)
    // }
    // pub fn accept_rate(ctx: Context<UpdateRate>) -> Result<()>{
    //     accept_rate_handler(ctx)
    // }
}