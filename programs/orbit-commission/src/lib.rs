use anchor_lang::prelude::*;

pub mod accessors;
pub mod structs;
pub mod errors;

pub use accessors::*;
pub use structs::*;
pub use errors::*;

// use crate::orbit_product::*;
// use crate::transactions::*;

declare_id!("9kTSzk3yQZBn2esehyucammd1WJ65GKcVAVCBGMdbcL5");

#[program]
pub mod orbit_commission_market {
    use super::*;
    use orbit_transaction::transaction_trait::OrbitTransactionTrait;
    use market_accounts::structs::OrbitMarketAccountTrait;

    //////////////////////////
    /// TRANSACTION

    /// SOL
    pub fn open_transaction_sol(ctx: Context<OpenCommissionTransactionSol>, seller_index: u8, buyer_index: u8, price: u64, use_discount: bool) -> Result<()>{
        CommissionTransaction::open_sol(ctx, seller_index, buyer_index, price, use_discount)
    }

    pub fn close_transaction_sol<'a>(ctx: Context<'_, '_, '_, 'a, CloseCommissionTransactionSol<'a>>) -> Result<()>{
        CommissionTransaction::close_sol(ctx)
    }

    pub fn fund_escrow_sol(ctx: Context<FundEscrowSol>) -> Result<()>{
        CommissionTransaction::fund_escrow_sol(ctx)
    }

    pub fn seller_early_decline_sol(ctx: Context<SellerEarlyDeclineSol>) -> Result<()>{
        CommissionTransaction::seller_early_decline_sol(ctx)
    }

    /// SPL
    pub fn open_transaction_spl(ctx: Context<OpenCommissionTransactionSpl>, seller_index: u8, buyer_index: u8,  price: u64, use_discount: bool) -> Result<()>{
        CommissionTransaction::open_spl(ctx, seller_index, buyer_index, price, use_discount)
    }

    pub fn close_transaction_spl<'a>(ctx: Context<'_, '_, '_, 'a, CloseCommissionTransactionSpl<'a>>) -> Result<()>{
        CommissionTransaction::close_spl(ctx)
    }

    pub fn fund_escrow_spl(ctx: Context<FundEscrowSpl>) -> Result<()>{
        CommissionTransaction::fund_escrow_spl(ctx)
    }
    
    pub fn seller_early_decline_spl(ctx: Context<SellerEarlyDeclineSpl>) -> Result<()>{
        CommissionTransaction::seller_early_decline_spl(ctx)
    }

    /// COMMON
    pub fn close_transaction_account(ctx: Context<CloseTransactionAccount>) -> Result<()>{
        CommissionTransaction::close_transaction_account(ctx)
    }
    
    /// BUYER UTILS
    pub fn confirm_delivered(ctx: Context<BuyerConfirmation>) -> Result<()>{
        confirm_delivered_handler(ctx)
    }
    
    pub fn confirm_accept(ctx: Context<BuyerConfirmation>) -> Result<()>{
        confirm_accept_handler(ctx)
    }
    
    pub fn deny_accept(ctx: Context<BuyerDeny>) -> Result<()>{
        deny_accept_handler(ctx)
    }

    /// SELLER UTILS
    pub fn commit_init_keys(ctx: Context<CommitInitData>, submission_keys: Vec<Pubkey>) -> Result<()>{
        commit_init_keys_handler(ctx, submission_keys)
    }

    pub fn commit_link(ctx: Context<CommitInitData>, link: String) -> Result<()>{
        commit_link_handler(ctx, link)
    }

    pub fn update_status_to_shipping(ctx: Context<CommitInitData>) -> Result<()>{
        update_status_to_shipping_handler(ctx)
    }

    pub fn commit_subkeys(ctx: Context<CommitSubKeys>, indexes: Vec<u8>) -> Result<()>{
        commit_subkeys_handler(ctx, indexes)
    }

    pub fn seller_accept_transaction(ctx: Context<SellerAcceptTransaction>) -> Result<()>{
        seller_accept_transaction_handler(ctx)
    }

    /// COMMISSION SPECIFIC UTILS
    
    pub fn commit_preview(ctx: Context<CommitPreview>, link: String) -> Result<()>{
        commit_preview_handler(ctx, link)
    }
    pub fn propose_rate(ctx: Context<UpdateRate>, new_rate: u8) -> Result<()>{
        propose_rate_handler(ctx, new_rate)
    }
    pub fn accept_rate(ctx: Context<UpdateRate>) -> Result<()>{
        accept_rate_handler(ctx)
    }

    //////////////////////////////
    /// PRODUCT
    

    /// MODIFIERS
    

    /////////////////////////////////////////////////
    /// REVIEW RELATED

    pub fn leave_review(ctx: Context<LeaveReview>, rating: u8) -> Result<()>{
        CommissionTransaction::leave_review(ctx, rating)
    }
}

