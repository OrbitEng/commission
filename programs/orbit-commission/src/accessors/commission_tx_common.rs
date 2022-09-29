use anchor_lang::prelude::*;
use market_accounts::OrbitMarketAccount;
use transaction::transaction_struct::TransactionState;
use crate::{DigitalTransaction, ComishAccount, DigitalMarketErrors};

////////////////////////////////////////////////////////////////////////////
/// COMMISSION PRODUCT HELPERS

#[derive(Accounts)]
pub struct CreateComishAccount<'info>{
    #[account(
        init,
        payer = payer,
        space = 250
    )]
    pub comish_account: Account<'info, ComishAccount>,

    #[account(
        mut,
        constraint = digital_transaction.metadata.transaction_state == TransactionState::BuyerFunded
    )]
    pub digital_transaction: Box<Account<'info, DigitalTransaction>>,

    #[account(
        constraint = (market_account.key() == digital_transaction.metadata.seller) || (market_account.key() == digital_transaction.metadata.buyer)
    )]
    pub market_account: Account<'info, OrbitMarketAccount>,

    #[account(
        mut,
        address = market_account.wallet
    )]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>
}

pub fn create_commission_account_handler(ctx: Context<CreateComishAccount>) -> Result<()>{
    ctx.accounts.comish_account.funder = ctx.accounts.payer.key();
    ctx.accounts.digital_transaction.has_comish = true;
    ctx.accounts.digital_transaction.comish_account = ctx.accounts.comish_account.key();
    Ok(())
}

//////////////////////////////////////////////////////////////////////////
/// COMMIT PREVIEW FROM SELLER

#[derive(Accounts)]
pub struct CommitPreview<'info>{
    #[account(
        has_one = comish_account,
        constraint = digital_transaction.metadata.transaction_state == TransactionState::BuyerFunded
    )]
    pub digital_transaction: Box<Account<'info, DigitalTransaction>>,

    #[account(mut)]
    pub comish_account: Account<'info, ComishAccount>,

    #[account(
        address = digital_transaction.metadata.seller,
        has_one = master_pubkey
    )]
    pub seller_account: Account<'info, OrbitMarketAccount>,

    pub master_pubkey: Signer<'info>,
}

pub fn commit_preview_handler(ctx: Context<CommitPreview>, link: [u8; 64]) -> Result<()>{
    ctx.accounts.comish_account.preview_address = link;
    Ok(())
}

////////////////////////////////////////////////////////////////////////////////////
/// RATE UPDATE UTILS FOR PREVIEW

#[derive(Accounts)]
pub struct UpdateRate<'info>{
    #[account(
        mut,
        has_one = comish_account,
        constraint = digital_transaction.metadata.transaction_state == TransactionState::BuyerFunded
    )]
    pub digital_transaction: Box<Account<'info, DigitalTransaction>>,

    #[account(mut)]
    pub comish_account: Account<'info, ComishAccount>,

    #[account(
        constraint = (market_account.key() == digital_transaction.metadata.seller) || (market_account.key() == digital_transaction.metadata.buyer),
        has_one = master_pubkey
    )]
    pub market_account: Account<'info, OrbitMarketAccount>,

    pub master_pubkey: Signer<'info>,
}

pub fn propose_rate_handler(ctx: Context<UpdateRate>, new_rate: u8) -> Result<()>{
    ctx.accounts.comish_account.preview_rate = new_rate;
    ctx.accounts.comish_account.last_rate_offerer = ctx.accounts.market_account.key();
    Ok(())
}

pub fn accept_rate_handler(ctx: Context<UpdateRate>) -> Result<()>{
    if ctx.accounts.market_account.key() == ctx.accounts.comish_account.last_rate_offerer{
        return err!(DigitalMarketErrors::InvalidRateAcceptor)
    };
    ctx.accounts.comish_account.last_rate_offerer = ctx.accounts.digital_transaction.metadata.seller;
    ctx.accounts.digital_transaction.close_rate = ctx.accounts.comish_account.preview_rate;
    Ok(())
}
