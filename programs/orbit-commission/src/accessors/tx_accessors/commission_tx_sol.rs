use anchor_lang::prelude::*;
use market_accounts::{
    OrbitMarketAccount,
    program::OrbitMarketAccounts
};
use orbit_product::program::OrbitProduct;
use orbit_product::CommissionProduct;
use crate::{
    CommissionTransaction,
    BuyerDecisionState, program::OrbitCommissionMarket,
};
use orbit_transaction::{transaction_struct::TransactionState, program::OrbitTransaction, BuyerOpenTransactions, SellerOpenTransactions};

#[derive(Accounts)]
#[instruction(seller_tx_index: u8)]
pub struct OpenCommissionTransactionSol<'info>{
    ////////////////////////////////
    /// TX
    #[account(
        init,
        payer = buyer_wallet,
        space = 2400,
        seeds = [
            b"orbit_commission_transaction",
            seller_transactions_log.key().as_ref(),
            [seller_tx_index].as_ref()
        ],
        bump
    )]
    pub commission_transaction: Box<Account<'info, CommissionTransaction>>,
    
    #[account(
        seeds = [
            b"orbit_escrow_account",
            commission_transaction.key().as_ref(),
            buyer_transactions_log.key().as_ref()
        ],
        bump
    )]
    pub escrow_account: SystemAccount<'info>,

    #[account(
        mut,
        constraint = commission_product.metadata.owner_catalog == seller_market_account.voter_id
    )] 
    pub commission_product: Box<Account<'info, CommissionProduct>>,
    
    //////////////////////////////////////////////////
    /// BUYER SELLER
    
    /// BUYER
    #[account(
        mut,
        seeds = [
            b"buyer_transactions",
            (&(orbit_transaction::TransactionType::Commissions).try_to_vec()?).as_slice(),
            &buyer_market_account.voter_id.to_le_bytes()
        ], 
        bump,
        seeds::program = &orbit_transaction::id()
    )]
    pub buyer_transactions_log: Box<Account<'info, BuyerOpenTransactions>>,

    #[account(
        mut
    )]
    pub buyer_market_account: Box<Account<'info, OrbitMarketAccount>>,

    #[account(
        mut,
        address = buyer_market_account.wallet
    )]
    pub buyer_wallet: Signer<'info>,
    
    /// SELLER

    #[account(    )]
    pub seller_market_account: Account<'info, OrbitMarketAccount>,

    #[account(
        mut,
        seeds = [
            b"seller_transactions",
            (&(orbit_transaction::TransactionType::Commissions).try_to_vec()?).as_slice(),
            &seller_market_account.voter_id.to_le_bytes()
        ], 
        bump,
        seeds::program = &orbit_transaction::id()
    )]
    pub seller_transactions_log: Box<Account<'info, SellerOpenTransactions>>,

    /////////////////////////////////
    /// EXTRANEOUS
    
    #[account(
        seeds = [b"market_authority"],
        bump
    )]
    pub commission_auth: SystemAccount<'info>,
    
    pub commission_program: Program<'info, OrbitCommissionMarket>,

    pub transaction_program: Program<'info, OrbitTransaction>,

    pub market_account_program: Program<'info, OrbitMarketAccounts>,
    
    pub product_program: Program<'info, OrbitProduct>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CloseCommissionTransactionSol<'info>{
    ////////////////////////////////////////////
    /// TX
    #[account(
        mut,
        constraint =    ((commission_transaction.metadata.transaction_state == TransactionState::BuyerConfirmedProduct) && (commission_transaction.final_decision != BuyerDecisionState::Null)) ||
                        (commission_transaction.metadata.transaction_state == TransactionState::Opened),
    )]
    pub commission_transaction: Box<Account<'info, CommissionTransaction>>,

    #[account(
        mut,
        constraint = commission_product.metadata.index == commission_transaction.metadata.product,
        constraint = commission_product.metadata.owner_catalog == seller_account.voter_id
    )] 
    pub commission_product: Box<Account<'info, CommissionProduct>>,

    #[account(
        mut,
        seeds = [
            b"orbit_escrow_account",
            commission_transaction.key().as_ref(),
            buyer_transactions_log.key().as_ref()
        ],
        bump
    )]
    pub escrow_account: SystemAccount<'info>,

    ///////////////////////////////////////////////////
    /// BUYER SELLER ACCOUNTS
    
    /// BUYER
    #[account(
        mut,
        constraint = buyer_account.voter_id == commission_transaction.metadata.buyer
    )]
    pub buyer_account: Box<Account<'info, OrbitMarketAccount>>,
    
    #[account(
        mut,
        seeds = [
            b"buyer_transactions",
            (&(orbit_transaction::TransactionType::Commissions).try_to_vec()?).as_slice(),
            &buyer_account.voter_id.to_le_bytes()
        ], 
        bump,
        seeds::program = &orbit_transaction::id()
    )]
    pub buyer_transactions_log: Box<Account<'info, BuyerOpenTransactions>>,

    #[account(
        mut,
        address = buyer_account.wallet
    )]
    pub buyer_wallet: SystemAccount<'info>,
    
    /// SELLER
    #[account(
        mut,
        constraint = seller_account.voter_id == commission_transaction.metadata.seller
    )]
    pub seller_account: Box<Account<'info, OrbitMarketAccount>>,

    #[account(
        mut,
        seeds = [
            b"seller_transactions",
            (&(orbit_transaction::TransactionType::Commissions).try_to_vec()?).as_slice(),
            &seller_account.voter_id.to_le_bytes()
        ], 
        bump,
        seeds::program = &orbit_transaction::id()
    )]
    pub seller_transactions_log: Box<Account<'info, SellerOpenTransactions>>,

    #[account(
        mut,
        address = seller_account.wallet
    )]
    pub seller_wallet: SystemAccount<'info>,

    //////////////////////////////////
    /// CPI AND EXTRANEOUS
    
    #[account(
        mut,
        address = Pubkey::new(orbit_addresses::MULTISIG_SIGNER)
    )]
    pub multisig_wallet: SystemAccount<'info>,

    #[account(
        seeds = [b"market_authority"],
        bump
    )]
    pub commission_auth: SystemAccount<'info>,

    pub commission_program: Program<'info, OrbitCommissionMarket>,
    
    pub market_account_program: Program<'info, OrbitMarketAccounts>,

    pub transaction_program: Program<'info, OrbitTransaction>,
    
    pub product_program: Program<'info, OrbitProduct>,
}

#[derive(Accounts)]
pub struct FundEscrowSol<'info>{
    ////////////////////////////////////////////
    /// TX
    
    #[account(
        mut,
        constraint = commission_transaction.metadata.transaction_state == TransactionState::SellerConfirmed
    )]
    pub commission_transaction: Box<Account<'info, CommissionTransaction>>,
    
    #[account(
        mut,
        seeds = [
            b"orbit_escrow_account",
            commission_transaction.key().as_ref(),
            buyer_transactions_log.key().as_ref()
        ],
        bump
    )]
    pub escrow_account: SystemAccount<'info>,

    ////////////////////////////////////////////
    /// BUYER SELLER

    /// BUYER
    #[account(
        mut,
        seeds = [
            b"buyer_transactions",
            (&(orbit_transaction::TransactionType::Commissions).try_to_vec()?).as_slice(),
            &buyer_market_account.voter_id.to_le_bytes()
        ], 
        bump,
        seeds::program = &orbit_transaction::id()
    )]
    pub buyer_transactions_log: Box<Account<'info, BuyerOpenTransactions>>,

    #[account(
        mut,
        constraint = buyer_market_account.voter_id == commission_transaction.metadata.buyer
    )]
    pub buyer_market_account: Box<Account<'info, OrbitMarketAccount>>,
    

    #[account(
        mut,
        address = buyer_market_account.wallet
    )]
    pub buyer_wallet: Signer<'info>
}

#[derive(Accounts)]
pub struct SellerEarlyDeclineSol<'info>{
    ////////////////////////////////////////////
    /// TX
    #[account(
        mut,
        constraint = commission_transaction.final_decision == BuyerDecisionState::Null
    )]
    pub commission_transaction: Box<Account<'info, CommissionTransaction>>,

    #[account(
        mut,
        seeds = [
            b"orbit_escrow_account",
            commission_transaction.key().as_ref(),
            buyer_transactions_log.key().as_ref()
        ],
        bump
    )]
    pub escrow_account: SystemAccount<'info>,

    ///////////////////////////////////////////////////
    /// BUYER SELLER ACCOUNTS
    
    /// BUYER
    #[account(
        mut,
        constraint = buyer_account.voter_id == commission_transaction.metadata.buyer
    )]
    pub buyer_account: Box<Account<'info, OrbitMarketAccount>>,
    
    /// BUYER
    #[account(
        mut,
        seeds = [
            b"buyer_transactions",
            (&(orbit_transaction::TransactionType::Commissions).try_to_vec()?).as_slice(),
            &buyer_account.voter_id.to_le_bytes()
        ], 
        bump,
        seeds::program = &orbit_transaction::id()
    )]
    pub buyer_transactions_log: Box<Account<'info, BuyerOpenTransactions>>,
    

    #[account(
        mut,
        address = buyer_account.wallet
    )]
    pub buyer_wallet: SystemAccount<'info>,
    
    /// SELLER
    #[account(
        mut,
        constraint = seller_account.voter_id == commission_transaction.metadata.seller
    )]
    pub seller_account: Box<Account<'info, OrbitMarketAccount>>,
    
    #[account(
        mut,
        seeds = [
            b"seller_transactions",
            (&(orbit_transaction::TransactionType::Commissions).try_to_vec()?).as_slice(),
            &seller_account.voter_id.to_le_bytes()
        ], 
        bump,
        seeds::program = &orbit_transaction::id()
    )]
    pub seller_transactions_log: Box<Account<'info, SellerOpenTransactions>>,

    #[account(
        mut,
        address = seller_account.wallet
    )]
    pub seller_wallet: Signer<'info>,

    //////////////////////////////////
    /// CPI AND EXTRANEOUS

    #[account(
        seeds = [b"market_authority"],
        bump
    )]
    pub commission_auth: SystemAccount<'info>,

    pub commission_program: Program<'info, OrbitCommissionMarket>,
    
    pub market_account_program: Program<'info, OrbitMarketAccounts>,

    pub transaction_program: Program<'info, OrbitTransaction>
}