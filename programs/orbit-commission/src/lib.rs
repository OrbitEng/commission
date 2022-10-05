use anchor_lang::prelude::*;
use product::product_struct::OrbitProduct;

pub mod accessors;
pub mod structs;
pub mod errors;

pub use accessors::*;
pub use structs::*;
pub use errors::*;

// use crate::product::*;
// use crate::transactions::*;

declare_id!("tS2LAkFMR7PVseGP4uLeSH49Htpv6JmYVFthBgviiFp");

#[program]
pub mod orbit_commission_market {
    use super::*;
    use transaction::transaction_trait::OrbitTransactionTrait;
    use market_accounts::structs::OrbitMarketAccountTrait;
    use product::product_trait::OrbitProductTrait;

    //////////////////////////
    /// INITIALIZATION
    
    pub fn init_commission_recent_catalog(ctx: Context<CreateCommissionRecentCatalog>) -> Result<()>{
        recent_commission_catalog_handler(ctx)
    }

    //////////////////////////
    /// TRANSACTION

    /// SOL
    pub fn open_transaction_sol(ctx: Context<OpenCommissionTransactionSol>, price: u64, use_discount: bool) -> Result<()>{
        CommissionTransaction::open_sol(ctx, price, use_discount)
    }

    pub fn close_transaction_sol<'a>(ctx: Context<'_, '_, '_, 'a, CloseCommissionTransactionSol<'a>>) -> Result<()>{
        CommissionTransaction::close_sol(ctx)
    }

    pub fn fund_escrow_sol(ctx: Context<FundEscrowSol>) -> Result<()>{
        CommissionTransaction::fund_escrow_sol(ctx)
    }

    /// SPL
    pub fn open_transaction_spl(ctx: Context<OpenCommissionTransactionSpl>, price: u64, use_discount: bool) -> Result<()>{
        CommissionTransaction::open_spl(ctx, price, use_discount)
    }

    pub fn close_transaction_spl<'a>(ctx: Context<'_, '_, '_, 'a, CloseCommissionTransactionSpl<'a>>) -> Result<()>{
        CommissionTransaction::close_spl(ctx)
    }

    pub fn fund_escrow_spl(ctx: Context<FundEscrowSpl>) -> Result<()>{
        CommissionTransaction::fund_escrow_spl(ctx)
    }

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
    
    pub fn deny_accept(ctx: Context<BuyerConfirmation>) -> Result<()>{
        deny_accept_handler(ctx)
    }

    pub fn early_decline(ctx: Context<EarlyDeclineTransaction>) -> Result<()>{
        early_decline_handler(ctx)
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
    
    pub fn list_product(ctx: Context<ListCommissionProduct>, prod: OrbitProduct)-> Result<()> {
        CommissionProduct::list(ctx, prod)
    }
    pub fn unlist_product(ctx: Context<UnlistCommissionProduct>)-> Result<()>{
        CommissionProduct::unlist(ctx)
    }

    /// MODIFIERS
    
    pub fn change_price(ctx: Context<UpdateProductField>, price: u64) -> Result<()>{
        update_price_handler(ctx, price)
    }
    pub fn update_currency(ctx: Context<UpdateProductField>, currency: Pubkey) -> Result<()>{
        update_currency_handler(ctx, currency)
    }
    pub fn set_media(ctx: Context<UpdateProductField>, link: String) -> Result<()>{
        update_media_handler(ctx, link)
    }
    pub fn set_prod_info(ctx: Context<UpdateProductField>, info: String) -> Result<()>{
        update_info_handler(ctx, info)
    }

    /////////////////////////////////////////////////
    /// REVIEW RELATED

    pub fn leave_review(ctx: Context<LeaveReview>, rating: u8) -> Result<()>{
        CommissionTransaction::leave_review(ctx, rating)
    }
}

