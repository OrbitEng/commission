use anchor_lang::{
    prelude::*,
    AccountsClose,
    solana_program::{
        program::{
            invoke,
            invoke_signed
        },
        system_instruction::transfer
    }
};
use orbit_transaction::{
    transaction_trait::OrbitTransactionTrait,
    transaction_struct::TransactionState,
    TransactionReviews,
    BuyerOpenTransactions,
    TransactionErrors, SellerOpenTransactions
};
use market_accounts::{
    market_account::OrbitMarketAccount, program::OrbitMarketAccounts,
    structs::market_account_trait::OrbitMarketAccountTrait,
    MarketAccountErrors,
    ReviewErrors,
    OrbitReflink
};
use anchor_spl::token::{
    accessor::amount,
    TokenAccount
};
use crate::{
    id,
    CommissionTransaction,

    CommissionMarketErrors,
    OpenCommissionTransactionSol,
    OpenCommissionTransactionSpl,
    CloseCommissionTransactionSol,
    CloseCommissionTransactionSpl,
    FundEscrowSol,
    FundEscrowSpl,
    BuyerDecisionState, program::OrbitCommissionMarket, SellerEarlyDeclineSpl, SellerEarlyDeclineSol
};

////////////////////////////////////////////////////////////////////
/// ORBIT BASE TRANSACTION FUNCTIONALITIES

#[derive(Accounts)]
pub struct CloseTransactionAccount<'info>{
    #[account(
        mut,
        constraint = commission_transaction.metadata.transaction_state == TransactionState::Closed,
    )]
    pub commission_transaction: Account<'info, CommissionTransaction>,

    #[account(
        has_one = wallet,
        constraint = 
        (proposer_account.voter_id == commission_transaction.metadata.seller) ||
        (proposer_account.voter_id == commission_transaction.metadata.buyer)
    )]
    pub proposer_account: Account<'info, OrbitMarketAccount>,
    
    pub wallet: Signer<'info>,

    #[account(
        constraint = buyer_account.voter_id == commission_transaction.metadata.buyer
    )]
    pub buyer_account: Account<'info, OrbitMarketAccount>,

    #[account(
        mut
    )]
    pub buyer_wallet: SystemAccount<'info>
}

impl<'a, 'b, 'c, 'd, 'e, 'f, 'g, 'h, 'i> OrbitTransactionTrait<'a, 'b, 'c, 'd, 'e, 'f, 'g, 'h, 'i, OpenCommissionTransactionSol<'a>, OpenCommissionTransactionSpl<'b>, CloseCommissionTransactionSol<'c>, CloseCommissionTransactionSpl<'d>, FundEscrowSol<'e>, FundEscrowSpl<'f>, CloseTransactionAccount<'g>, SellerEarlyDeclineSol<'h>, SellerEarlyDeclineSpl<'i>> for CommissionTransaction{
    fn open_sol(ctx: Context<OpenCommissionTransactionSol>, seller_index: u8, buyer_index: u8, mut price: u64, use_discount: bool) -> Result<()>{
        let auth_bump: &u8;
        if let Some(ab) = ctx.bumps.get("commission_auth"){
            auth_bump = ab
        }else{
            return err!(CommissionMarketErrors::InvalidAuthBump)
        };
        if use_discount && ctx.accounts.buyer_market_account.dispute_discounts > 0{
            ctx.accounts.commission_transaction.metadata.rate = 100;
            price = price * 95 / 100;
            
            market_accounts::cpi::decrement_dispute_discounts(
                CpiContext::new_with_signer(
                    ctx.accounts.market_account_program.to_account_info(),
                    market_accounts::cpi::accounts::MarketAccountUpdateInternal{
                        market_account: ctx.accounts.buyer_market_account.to_account_info(),
                        caller_auth: ctx.accounts.commission_auth.to_account_info(),
                        caller: ctx.accounts.commission_program.to_account_info()
                    },
                    &[&[b"market_authority", &[*auth_bump]]]
                )
            )?;
        }else{
            ctx.accounts.commission_transaction.metadata.rate = 95
        }
        ctx.accounts.commission_transaction.metadata.buyer = ctx.accounts.buyer_market_account.voter_id;
        ctx.accounts.commission_transaction.metadata.seller = ctx.accounts.seller_market_account.voter_id;
        ctx.accounts.commission_transaction.metadata.product = ctx.accounts.commission_product.metadata.index;
        ctx.accounts.commission_transaction.metadata.transaction_state = TransactionState::Opened;
        ctx.accounts.commission_transaction.metadata.transaction_price = price;
        ctx.accounts.commission_transaction.metadata.funded = false;
        ctx.accounts.commission_transaction.metadata.currency = System::id();

        ctx.accounts.commission_transaction.num_keys = 0;
        ctx.accounts.commission_transaction.final_decision = BuyerDecisionState::Null;

        ctx.accounts.commission_transaction.metadata.reviews = TransactionReviews{
            buyer: false,
            seller: false
        };

        orbit_transaction::cpi::add_buyer_commissions_transaction(
            CpiContext::new_with_signer(
                ctx.accounts.transaction_program.to_account_info(),
                orbit_transaction::cpi::accounts::AddBuyerCommissionsTransactions{
                    buyer_account: ctx.accounts.buyer_market_account.to_account_info(),
                    wallet: ctx.accounts.buyer_wallet.to_account_info(),
                    transactions_log: ctx.accounts.buyer_transactions_log.to_account_info(),
                    tx: ctx.accounts.commission_transaction.to_account_info()
                },
                &[&[b"market_authority", &[*auth_bump]]]
            ),
            buyer_index
        )?;
        orbit_transaction::cpi::add_seller_commissions_transaction(
            CpiContext::new(
                ctx.accounts.transaction_program.to_account_info(),
                orbit_transaction::cpi::accounts::AddSellerCommissionsTransactions{
                    transactions_log: ctx.accounts.seller_transactions_log.to_account_info(),
                    tx: ctx.accounts.commission_transaction.to_account_info()
                }
            ),
            seller_index
        )?;

        Ok(())
    }

    fn open_spl(ctx: Context<OpenCommissionTransactionSpl>, seller_index: u8, buyer_index: u8, mut price: u64, use_discount: bool) -> Result<()>{
        let auth_bump: &u8;
        if let Some(ab) = ctx.bumps.get("commission_auth"){
            auth_bump = ab
        }else{
            return err!(CommissionMarketErrors::InvalidAuthBump)
        };
        if use_discount && ctx.accounts.buyer_market_account.dispute_discounts > 0{
            ctx.accounts.commission_transaction.metadata.rate = 100;
            price = price * 95 / 100;
            
            market_accounts::cpi::decrement_dispute_discounts(
                CpiContext::new_with_signer(
                    ctx.accounts.market_account_program.to_account_info(),
                    market_accounts::cpi::accounts::MarketAccountUpdateInternal{
                        market_account: ctx.accounts.buyer_market_account.to_account_info(),
                        caller_auth: ctx.accounts.commission_auth.to_account_info(),
                        caller: ctx.accounts.commission_program.to_account_info()
                    },
                    &[&[b"market_authority", &[*auth_bump]]]
                )
            )?;
        }else{
            ctx.accounts.commission_transaction.metadata.rate = 95
        }
        ctx.accounts.commission_transaction.metadata.buyer = ctx.accounts.buyer_market_account.voter_id;
        ctx.accounts.commission_transaction.metadata.seller = ctx.accounts.seller_market_account.voter_id;
        ctx.accounts.commission_transaction.metadata.product = ctx.accounts.commission_product.metadata.index;
        ctx.accounts.commission_transaction.metadata.transaction_state = TransactionState::Opened;
        ctx.accounts.commission_transaction.metadata.transaction_price = price;
        ctx.accounts.commission_transaction.metadata.funded = false;
        ctx.accounts.commission_transaction.metadata.currency = ctx.accounts.token_mint.key();
        
        ctx.accounts.commission_transaction.num_keys = 0;
        ctx.accounts.commission_transaction.final_decision = BuyerDecisionState::Null;

        ctx.accounts.commission_transaction.metadata.reviews = TransactionReviews{
            buyer: false,
            seller: false
        };

        orbit_transaction::cpi::add_buyer_commissions_transaction(
            CpiContext::new_with_signer(
                ctx.accounts.transaction_program.to_account_info(),
                orbit_transaction::cpi::accounts::AddBuyerCommissionsTransactions{
                    buyer_account: ctx.accounts.buyer_market_account.to_account_info(),
                    wallet: ctx.accounts.buyer_wallet.to_account_info(),
                    transactions_log: ctx.accounts.buyer_transactions_log.to_account_info(),
                    tx: ctx.accounts.commission_transaction.to_account_info()
                },
                &[&[b"market_authority", &[*auth_bump]]]
            ),
            buyer_index
        )?;
        orbit_transaction::cpi::add_seller_commissions_transaction(
            CpiContext::new(
                ctx.accounts.transaction_program.to_account_info(),
                orbit_transaction::cpi::accounts::AddSellerCommissionsTransactions{
                    transactions_log: ctx.accounts.seller_transactions_log.to_account_info(),
                    tx: ctx.accounts.commission_transaction.to_account_info()
                }
            ),
            seller_index
        )?;
        Ok(())
    }

    fn close_sol(ctx: Context<'_, '_, '_, 'c, CloseCommissionTransactionSol<'c>>) -> Result<()>{
        let comm_tx = ctx.accounts.commission_transaction.key();
        let comm_seed = comm_tx.as_ref();
        let buyer_log = ctx.accounts.buyer_transactions_log.key();
        let buyer_tx_log_seed = buyer_log.as_ref();

        if let Some(escrow_bump) = ctx.bumps.get("escrow_account"){
            if (ctx.accounts.commission_transaction.close_rate == 95)
                && (ctx.accounts.commission_transaction.final_decision == BuyerDecisionState::Accept){
                    let bal = ctx.accounts.escrow_account.lamports();
                    let mut residual_amt = bal * 5/100;
                    if  (ctx.accounts.buyer_account.used_reflink != Pubkey::new(&[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0])) &&
                        (ctx.remaining_accounts[0].key() == ctx.accounts.buyer_account.used_reflink)
                    {
                        let reflink_amt = bal * 25 / 10000;
                        residual_amt = bal * 45/1000;
                        orbit_transaction::close_escrow_sol_flat!(
                            ctx.accounts.escrow_account.to_account_info(),
                            ctx.accounts.buyer_wallet.to_account_info(),
                            &[&[b"orbit_escrow_account", comm_seed, buyer_tx_log_seed, &[*escrow_bump]]],
                            reflink_amt
                        ).expect("couldnt close escrow");
                        match orbit_transaction::remaining_accounts_to_wallet!(ctx.remaining_accounts){
                            Ok(reflink_wallet) => {
                                orbit_transaction::close_escrow_sol_flat!(
                                    ctx.accounts.escrow_account.to_account_info(),
                                    reflink_wallet.to_account_info(),
                                    &[&[b"orbit_escrow_account", comm_seed, buyer_tx_log_seed, &[*escrow_bump]]],
                                    reflink_amt
                                ).expect("couldnt close escrow");
                                reflink_wallet.exit(ctx.program_id)?;
                            },
                            Err(e) => return Err(e)
                        }
                    }
                    orbit_transaction::close_escrow_sol_flat!(
                        ctx.accounts.escrow_account.to_account_info(),
                        ctx.accounts.multisig_wallet.to_account_info(),
                        &[&[b"orbit_escrow_account", comm_seed, buyer_tx_log_seed, &[*escrow_bump]]],
                        residual_amt
                    ).expect("couldnt close escrow");
                }

                orbit_transaction::close_escrow_sol_rate!(
                    ctx.accounts.escrow_account.to_account_info(),
                    ctx.accounts.seller_wallet.to_account_info(),
                    &[&[b"orbit_escrow_account", comm_seed, buyer_tx_log_seed, &[*escrow_bump]]],
                    ctx.accounts.commission_transaction.close_rate
                ).expect("could not transfer tokens");
                orbit_transaction::close_escrow_sol_rate!(
                    ctx.accounts.escrow_account.to_account_info(),
                    ctx.accounts.buyer_wallet.to_account_info(),
                    &[&[b"orbit_escrow_account", comm_seed, buyer_tx_log_seed, &[*escrow_bump]]],
                    100
                ).expect("could not transfer tokens");
            
            
        }else{
            return err!(CommissionMarketErrors::InvalidEscrowBump)
        };

        if let Some(auth_bump) = ctx.bumps.get("commission_auth"){
            orbit_transaction::post_tx_incrementing!(
                ctx.accounts.market_account_program.to_account_info(),
                ctx.accounts.buyer_account.to_account_info(),
                ctx.accounts.seller_account.to_account_info(),
                ctx.accounts.commission_auth.to_account_info(),
                ctx.accounts.commission_program.to_account_info(),
                &[&[b"market_authority", &[*auth_bump]]]
            )
        }else{
            return err!(CommissionMarketErrors::InvalidAuthBump)
        }?;

        orbit_transaction::cpi::clear_seller_commissions_transaction(
            CpiContext::new(
                ctx.accounts.transaction_program.to_account_info(),
                orbit_transaction::cpi::accounts::ClearSellerCommissionsTransactions{
                    transactions_log: ctx.accounts.seller_transactions_log.to_account_info(),
                    caller_auth: ctx.accounts.commission_auth.to_account_info(),
                    caller: ctx.accounts.commission_program.to_account_info()
                }
            ),
            ctx.accounts.commission_transaction.metadata.seller_tx_index
        )?;

        orbit_transaction::cpi::clear_buyer_commissions_transaction(
            CpiContext::new(
                ctx.accounts.transaction_program.to_account_info(),
                orbit_transaction::cpi::accounts::ClearBuyerCommissionsTransactions{
                    transactions_log: ctx.accounts.buyer_transactions_log.to_account_info(),
                    caller_auth: ctx.accounts.commission_auth.to_account_info(),
                    caller: ctx.accounts.commission_program.to_account_info()
                }
            ),
            ctx.accounts.commission_transaction.metadata.seller_tx_index
        )?;

        ctx.accounts.commission_transaction.metadata.transaction_state = TransactionState::Closed;
        Ok(())
    }

    fn close_spl(ctx: Context<'_, '_, '_, 'd, CloseCommissionTransactionSpl<'d>>) -> Result<()>{
        if let Some(auth_bump) = ctx.bumps.get("commission_auth"){
            if (ctx.accounts.commission_transaction.close_rate == 95)
                && (ctx.accounts.commission_transaction.final_decision == BuyerDecisionState::Accept){
                    let bal = amount(&ctx.accounts.escrow_account.to_account_info()).expect("could not deserialize token account");
                    let mut residual_amt = bal * 5/100;
                    if  (ctx.accounts.buyer_account.used_reflink != Pubkey::new(&[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0])) &&
                        (ctx.remaining_accounts[0].key() == ctx.accounts.buyer_account.used_reflink)
                    {
                        let reflink_amt = bal * 25 / 10000;
                        residual_amt = bal * 45/1000;
                        orbit_transaction::close_escrow_spl_flat!(
                            ctx.accounts.token_program.to_account_info(),
                            ctx.accounts.escrow_account.to_account_info(),
                            ctx.accounts.buyer_token_account.to_account_info(),
                            ctx.accounts.commission_auth.to_account_info(),
                            &[&[b"market_authority", &[*auth_bump]]],
                            reflink_amt
                        ).expect("couldnt close escrow");
                        

                        match orbit_transaction::remaining_accounts_to_token_account!(ctx.remaining_accounts){
                            Ok(reflink_token_account) => {
                                orbit_transaction::close_escrow_spl_flat!(
                                    ctx.accounts.token_program.to_account_info(),
                                    ctx.accounts.escrow_account.to_account_info(),
                                    reflink_token_account.to_account_info(),
                                    ctx.accounts.commission_auth.to_account_info(),
                                    &[&[b"market_authority", &[*auth_bump]]],
                                    reflink_amt
                                ).expect("couldnt close escrow");
                                reflink_token_account.exit(ctx.program_id)?;
                            },
                            Err(e) => return Err(e)
                        }
                    }
                    orbit_transaction::close_escrow_spl_flat!(
                        ctx.accounts.token_program.to_account_info(),
                        ctx.accounts.escrow_account.to_account_info(),
                        ctx.accounts.multisig_ata.to_account_info(),
                        ctx.accounts.commission_auth.to_account_info(),
                        &[&[b"market_authority", &[*auth_bump]]],
                        residual_amt
                    ).expect("couldnt close escrow");
                }

                orbit_transaction::close_escrow_spl_rate!(
                    ctx.accounts.token_program.to_account_info(),
                    ctx.accounts.escrow_account.to_account_info(),
                    ctx.accounts.seller_token_account.to_account_info(),
                    ctx.accounts.commission_auth.to_account_info(),
                    &[&[b"market_authority", &[*auth_bump]]],
                    ctx.accounts.commission_transaction.metadata.transaction_price,
                    ctx.accounts.commission_transaction.close_rate
                )?;
                orbit_transaction::close_escrow_spl_rate!(
                    ctx.accounts.token_program.to_account_info(),
                    ctx.accounts.escrow_account.to_account_info(),
                    ctx.accounts.buyer_token_account.to_account_info(),
                    ctx.accounts.commission_auth.to_account_info(),
                    &[&[b"market_authority", &[*auth_bump]]],
                    ctx.accounts.commission_transaction.metadata.transaction_price,
                    100
                )?;
                orbit_transaction::post_tx_incrementing!(
                    ctx.accounts.market_account_program.to_account_info(),
                    ctx.accounts.buyer_account.to_account_info(),
                    ctx.accounts.seller_account.to_account_info(),
                    ctx.accounts.commission_auth.to_account_info(),
                    ctx.accounts.commission_program.to_account_info(),
                    &[&[b"market_authority", &[*auth_bump]]]
                )
        }else{
            return err!(CommissionMarketErrors::InvalidAuthBump)
        }?;

        orbit_transaction::cpi::clear_seller_commissions_transaction(
            CpiContext::new(
                ctx.accounts.transaction_program.to_account_info(),
                orbit_transaction::cpi::accounts::ClearSellerCommissionsTransactions{
                    transactions_log: ctx.accounts.seller_transactions_log.to_account_info(),
                    caller_auth: ctx.accounts.commission_auth.to_account_info(),
                    caller: ctx.accounts.commission_program.to_account_info()
                }
            ),
            ctx.accounts.commission_transaction.metadata.seller_tx_index
        )?;

        orbit_transaction::cpi::clear_buyer_commissions_transaction(
            CpiContext::new(
                ctx.accounts.transaction_program.to_account_info(),
                orbit_transaction::cpi::accounts::ClearBuyerCommissionsTransactions{
                    transactions_log: ctx.accounts.buyer_transactions_log.to_account_info(),
                    caller_auth: ctx.accounts.commission_auth.to_account_info(),
                    caller: ctx.accounts.commission_program.to_account_info()
                }
            ),
            ctx.accounts.commission_transaction.metadata.seller_tx_index
        )?;

        ctx.accounts.commission_transaction.metadata.transaction_state = TransactionState::Closed;
        Ok(())
    }

    fn fund_escrow_sol(ctx: Context<FundEscrowSol>) -> Result<()>{
        invoke(
            &transfer(
                &ctx.accounts.buyer_wallet.key(),
                &ctx.accounts.escrow_account.key(),
                ctx.accounts.commission_transaction.metadata.transaction_price
            ),
            &[
                ctx.accounts.buyer_wallet.to_account_info(),
                ctx.accounts.escrow_account.to_account_info()
            ]
        ).expect("could not fund escrow");
        ctx.accounts.commission_transaction.metadata.funded = true;
        ctx.accounts.commission_transaction.metadata.transaction_state = TransactionState::BuyerFunded;
        Ok(())
    }

    fn fund_escrow_spl(ctx: Context<FundEscrowSpl>) -> Result<()>{
        anchor_spl::token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(), 
                anchor_spl::token::Transfer{
                    from: ctx.accounts.buyer_token_account.to_account_info(),
                    to: ctx.accounts.escrow_account.to_account_info(),
                    authority: ctx.accounts.buyer_wallet.to_account_info()
                }
            ),
            ctx.accounts.commission_transaction.metadata.transaction_price
        ).expect("could not fund escrow account. maybe check your balance");
        ctx.accounts.commission_transaction.metadata.funded = true;
        ctx.accounts.commission_transaction.metadata.transaction_state = TransactionState::BuyerFunded;
        Ok(())
    }

    fn close_transaction_account(ctx: Context<CloseTransactionAccount>) -> Result<()>{
        ctx.accounts.commission_transaction.close(ctx.accounts.buyer_wallet.to_account_info())
    }

    fn seller_early_decline_sol(ctx: Context<SellerEarlyDeclineSol>) -> Result<()>{
        ctx.accounts.commission_transaction.metadata.transaction_state = TransactionState::Closed;
        if ctx.accounts.commission_transaction.metadata.rate == 100{
            market_accounts::cpi::increment_dispute_discounts(
                CpiContext::new(
                    ctx.accounts.market_account_program.to_account_info(),
                    market_accounts::cpi::accounts::MarketAccountUpdateInternal{
                        market_account: ctx.accounts.buyer_account.to_account_info(),
                        caller_auth: ctx.accounts.commission_auth.to_account_info(),
                        caller: ctx.accounts.commission_program.to_account_info()
                    }
                )
            )?;
        };

        let comm_tx = ctx.accounts.commission_transaction.key();
        let comm_seed = comm_tx.as_ref();
        let buyer_log = ctx.accounts.buyer_transactions_log.key();
        let buyer_tx_log_seed = buyer_log.as_ref();

        if let Some(escrow_bump) = ctx.bumps.get("escrow_account"){
            orbit_transaction::close_escrow_sol_rate!(
                ctx.accounts.escrow_account.to_account_info(),
                ctx.accounts.seller_wallet.to_account_info(),
                &[&[b"orbit_escrow_account", comm_seed, buyer_tx_log_seed, &[*escrow_bump]]],
                ctx.accounts.commission_transaction.close_rate
            ).expect("could not transfer tokens");
            orbit_transaction::close_escrow_sol_rate!(
                ctx.accounts.escrow_account.to_account_info(),
                ctx.accounts.buyer_wallet.to_account_info(),
                &[&[b"orbit_escrow_account", comm_seed, buyer_tx_log_seed, &[*escrow_bump]]],
                100
            ).expect("could not transfer tokens");
        }else{
            return err!(CommissionMarketErrors::InvalidEscrowBump)
        };

        if let Some(escrow_seeds) = ctx.bumps.get("escrow_account"){
            orbit_transaction::close_escrow_sol_rate!(
                ctx.accounts.escrow_account.to_account_info(),
                ctx.accounts.buyer_wallet.to_account_info(),
                &[&[b"orbit_escrow_account", comm_seed, buyer_tx_log_seed, &[*escrow_seeds]]],
                100
            )?;
        }else{
            return err!(CommissionMarketErrors::InvalidEscrowBump)
        };
        
        orbit_transaction::cpi::clear_seller_commissions_transaction(
            CpiContext::new(
                ctx.accounts.transaction_program.to_account_info(),
                orbit_transaction::cpi::accounts::ClearSellerCommissionsTransactions{
                    transactions_log: ctx.accounts.seller_transactions_log.to_account_info(),
                    caller_auth: ctx.accounts.commission_auth.to_account_info(),
                    caller: ctx.accounts.commission_program.to_account_info()
                }
            ),
            ctx.accounts.commission_transaction.metadata.seller_tx_index
        )?;

        orbit_transaction::cpi::clear_buyer_commissions_transaction(
            CpiContext::new(
                ctx.accounts.transaction_program.to_account_info(),
                orbit_transaction::cpi::accounts::ClearBuyerCommissionsTransactions{
                    transactions_log: ctx.accounts.buyer_transactions_log.to_account_info(),
                    caller_auth: ctx.accounts.commission_auth.to_account_info(),
                    caller: ctx.accounts.commission_program.to_account_info()
                }
            ),
            ctx.accounts.commission_transaction.metadata.seller_tx_index
        )?;
        Ok(())
    }

    fn seller_early_decline_spl(ctx: Context<SellerEarlyDeclineSpl>) -> Result<()>{
        ctx.accounts.commission_transaction.metadata.transaction_state = TransactionState::Closed;

        if ctx.accounts.commission_transaction.metadata.rate == 100{
            market_accounts::cpi::increment_dispute_discounts(
                CpiContext::new(
                    ctx.accounts.market_account_program.to_account_info(),
                    market_accounts::cpi::accounts::MarketAccountUpdateInternal{
                        market_account: ctx.accounts.buyer_account.to_account_info(),
                        caller_auth: ctx.accounts.commission_auth.to_account_info(),
                        caller: ctx.accounts.commission_program.to_account_info()
                    }
                )
            )?;
        }

        if let Some(auth_bump) = ctx.bumps.get("commission_auth"){
            orbit_transaction::close_escrow_spl_rate!(
                ctx.accounts.token_program.to_account_info(),
                ctx.accounts.escrow_account.to_account_info(),
                ctx.accounts.seller_token_account.to_account_info(),
                ctx.accounts.commission_auth.to_account_info(),
                &[&[b"market_authority", &[*auth_bump]]],
                ctx.accounts.commission_transaction.metadata.transaction_price,
                ctx.accounts.commission_transaction.close_rate
            ).expect("could not transfer tokens");
            orbit_transaction::close_escrow_spl_rate!(
                ctx.accounts.token_program.to_account_info(),
                ctx.accounts.escrow_account.to_account_info(),
                ctx.accounts.buyer_token_account.to_account_info(),
                ctx.accounts.commission_auth.to_account_info(),
                &[&[b"market_authority", &[*auth_bump]]],
                ctx.accounts.commission_transaction.metadata.transaction_price,
                100
            ).expect("could not transfer tokens");
        }else{
            return err!(CommissionMarketErrors::InvalidAuthBump)
        }; 
        
        orbit_transaction::cpi::clear_seller_commissions_transaction(
            CpiContext::new(
                ctx.accounts.transaction_program.to_account_info(),
                orbit_transaction::cpi::accounts::ClearSellerCommissionsTransactions{
                    transactions_log: ctx.accounts.seller_transactions_log.to_account_info(),
                    caller_auth: ctx.accounts.commission_auth.to_account_info(),
                    caller: ctx.accounts.commission_program.to_account_info()
                }
            ),
            ctx.accounts.commission_transaction.metadata.seller_tx_index
        )?;

        orbit_transaction::cpi::clear_buyer_commissions_transaction(
            CpiContext::new(
                ctx.accounts.transaction_program.to_account_info(),
                orbit_transaction::cpi::accounts::ClearBuyerCommissionsTransactions{
                    transactions_log: ctx.accounts.buyer_transactions_log.to_account_info(),
                    caller_auth: ctx.accounts.commission_auth.to_account_info(),
                    caller: ctx.accounts.commission_program.to_account_info()
                }
            ),
            ctx.accounts.commission_transaction.metadata.seller_tx_index
        )?;

        Ok(())

    }

}

//////////////////////////////////////////////////////////////////////////
/// BUYER CONFIRMATIONS

#[derive(Accounts)]
pub struct BuyerConfirmation<'info>{
    #[account(
        mut,
        constraint = commission_transaction.final_decision == BuyerDecisionState::Null,
    )]
    pub commission_transaction: Box<Account<'info, CommissionTransaction>>,

    #[account(
        constraint = buyer_market_account.voter_id == commission_transaction.metadata.buyer
    )]
    pub buyer_market_account: Account<'info, OrbitMarketAccount>,

    #[account(
        seeds = [
            b"buyer_transactions",
            (&(orbit_transaction::TransactionType::Commissions).try_to_vec()?).as_slice(),
            &buyer_market_account.voter_id.to_le_bytes()
        ], 
        bump,
        seeds::program = &orbit_transaction::id()
    )]
    pub buyer_transactions: Box<Account<'info, BuyerOpenTransactions>>,

    pub buyer_wallet: Signer<'info>,
}

pub fn confirm_delivered_handler(ctx: Context<BuyerConfirmation>) -> Result<()>{
    if ctx.accounts.commission_transaction.metadata.transaction_state != TransactionState::Shipped{
        return err!(CommissionMarketErrors::WaitingForSellerData);
    }
    ctx.accounts.commission_transaction.metadata.transaction_state = TransactionState::BuyerConfirmedDelivery;

    Ok(())
}

pub fn confirm_accept_handler(ctx: Context<BuyerConfirmation>) -> Result<()>{
    if ctx.accounts.commission_transaction.metadata.transaction_state != TransactionState::BuyerConfirmedDelivery{
        return err!(CommissionMarketErrors::DidNotConfirmDelivery);
    }
    ctx.accounts.commission_transaction.final_decision = BuyerDecisionState::Accept;
    ctx.accounts.commission_transaction.close_rate = ctx.accounts.commission_transaction.metadata.rate;
    // we dont set state here because we need to wait for the seller to release the final keys
    Ok(())
}

#[derive(Accounts)]
pub struct BuyerDeny<'info>{
    #[account(
        mut,
        constraint = commission_transaction.final_decision == BuyerDecisionState::Null,
    )]
    pub commission_transaction: Box<Account<'info, CommissionTransaction>>,

    #[account(
        mut
    )]
    pub buyer_account: Account<'info, OrbitMarketAccount>,

    #[account(
        seeds = [
            b"buyer_transactions",
            (&(orbit_transaction::TransactionType::Commissions).try_to_vec()?).as_slice(),
            &buyer_account.voter_id.to_le_bytes()
        ], 
        bump,
        seeds::program = &orbit_transaction::id()
    )]
    pub buyer_transactions: Box<Account<'info, BuyerOpenTransactions>>,

    #[account(
        address = buyer_account.wallet
    )]
    pub buyer_wallet: Signer<'info>,
    
    #[account(
        seeds = [b"market_authority"],
        bump
    )]
    pub commission_auth: SystemAccount<'info>,

    pub commission_program: Program<'info, OrbitCommissionMarket>,

    pub market_accounts_program: Program<'info, OrbitMarketAccounts>
}

pub fn deny_accept_handler(ctx: Context<BuyerDeny>) -> Result<()>{
    if ctx.accounts.commission_transaction.metadata.transaction_state != TransactionState::BuyerConfirmedDelivery{
        return err!(CommissionMarketErrors::DidNotConfirmDelivery);
    }
    if ctx.accounts.commission_transaction.metadata.rate == 100{
        market_accounts::cpi::increment_dispute_discounts(
            CpiContext::new(
                ctx.accounts.market_accounts_program.to_account_info(),
                market_accounts::cpi::accounts::MarketAccountUpdateInternal{
                    market_account: ctx.accounts.buyer_account.to_account_info(),
                    caller_auth: ctx.accounts.commission_auth.to_account_info(),
                    caller: ctx.accounts.commission_program.to_account_info()
                }
            )
        )?;
    }
    ctx.accounts.commission_transaction.metadata.rate = 0;
    ctx.accounts.commission_transaction.close_rate = 0;
    ctx.accounts.commission_transaction.final_decision = BuyerDecisionState::Declined;
    ctx.accounts.commission_transaction.metadata.transaction_state = TransactionState::BuyerConfirmedProduct;

    Ok(())
}

///////////////////////////////////////////////////////////////////////
/// SELLER CONFIRMATIONS

#[derive(Accounts)]
pub struct SellerAcceptTransaction<'info>{
    #[account(
        mut,
        constraint = commission_transaction.metadata.transaction_state == TransactionState::Opened
    )]
    pub commission_transaction: Box<Account<'info, CommissionTransaction>>,

    #[account(
        constraint = seller_market_account.voter_id == commission_transaction.metadata.seller
    )]
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
    pub seller_transactions: Box<Account<'info, SellerOpenTransactions>>,

    #[account(
        address = seller_market_account.wallet
    )]
    pub wallet: Signer<'info>
}

pub fn seller_accept_transaction_handler(ctx: Context<SellerAcceptTransaction>) -> Result<()>{
    ctx.accounts.commission_transaction.metadata.transaction_state = TransactionState::SellerConfirmed;
    Ok(())
}

#[derive(Accounts)]
pub struct CommitInitData<'info>{
    #[account(
        mut,
        constraint = commission_transaction.metadata.transaction_state == TransactionState::BuyerFunded
    )]
    pub commission_transaction: Box<Account<'info, CommissionTransaction>>,

    #[account(
        constraint = seller_market_account.voter_id == commission_transaction.metadata.seller
    )]
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
    pub seller_transactions: Box<Account<'info, SellerOpenTransactions>>,

    #[account(
        address = seller_market_account.wallet
    )]
    pub seller_wallet: Signer<'info>,
}

pub fn commit_init_keys_handler(ctx: Context<CommitInitData>, submission_keys: Vec<Pubkey>) -> Result<()>{   
    if submission_keys.len() > 64{
        return err!(CommissionMarketErrors::IndexOutOfRange)
    }

    ctx.accounts.commission_transaction.num_keys = submission_keys.len() as u64;
    ctx.accounts.commission_transaction.key_arr = submission_keys;
    Ok(())
}

pub fn commit_link_handler(ctx: Context<CommitInitData>, link: String) -> Result<()>{
    ctx.accounts.commission_transaction.data_address = link;
    Ok(())
}

pub fn update_status_to_shipping_handler(ctx: Context<CommitInitData>) -> Result<()>{
    ctx.accounts.commission_transaction.metadata.transaction_state = TransactionState::Shipped;
    Ok(())
}

#[derive(Accounts)]
pub struct CommitSubKeys<'info>{
    #[account(
        mut,
        constraint = commission_transaction.metadata.transaction_state == TransactionState::BuyerConfirmedDelivery
    )]
    pub commission_transaction: Box<Account<'info, CommissionTransaction>>,

    #[account(
        constraint = seller_market_account.voter_id == commission_transaction.metadata.seller
    )]
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
    pub seller_transactions: Box<Account<'info, SellerOpenTransactions>>,

    #[account(
        address = seller_market_account.wallet
    )]
    pub seller_wallet: Signer<'info>,
}

pub fn commit_subkeys_handler(ctx: Context<CommitSubKeys>, indexes: Vec<u8>) -> Result<()>{
    for index in indexes{
        if index > ctx.accounts.commission_transaction.key_arr.len() as u8{
            return err!(CommissionMarketErrors::IndexOutOfRange)
        }

        let acc = &ctx.remaining_accounts[index as usize];
        if ! acc.is_signer{
            return err!(CommissionMarketErrors::CorruptPrivateKeyFormat);
        }

        if Pubkey::find_program_address(&[acc.key().as_ref()], &id()).0 != ctx.accounts.commission_transaction.key_arr[index as usize]{
            return err!(CommissionMarketErrors::IncorrectPrivateKey);
        }

        ctx.accounts.commission_transaction.num_keys &= u64::MAX - (1 << index);
        ctx.accounts.commission_transaction.key_arr[index as usize] = acc.key();
    }

    if ctx.accounts.commission_transaction.num_keys == 0{
        ctx.accounts.commission_transaction.metadata.transaction_state = TransactionState::BuyerConfirmedProduct;
    }

    Ok(())
}

///////////////////////////////////////////////////////////////////////
/// ACCOUNT HELPERS (leave a review!)

#[derive(Accounts)]
pub struct LeaveReview<'info>{
    /////////////////////////////////////////////////
    /// TX
    #[account(
        mut,
        constraint = commission_transaction.metadata.transaction_state == TransactionState::Closed
    )]
    pub commission_transaction: Box<Account<'info, CommissionTransaction>>,

    /////////////////////////////////////////////////
    /// REVIEW RELATED
    #[account(
        mut,
        constraint = 
        (reviewer.voter_id == commission_transaction.metadata.seller) ||
        (reviewer.voter_id == commission_transaction.metadata.buyer)
    )]
    pub reviewed_account: Box<Account<'info, OrbitMarketAccount>>,

    #[account(
        constraint = 
        (reviewer.voter_id == commission_transaction.metadata.seller) ||
        (reviewer.voter_id == commission_transaction.metadata.buyer),
        seeds = [
            b"orbit_account",
            wallet.key().as_ref()
        ],
        bump,
        seeds::program = market_accounts::ID
    )]
    pub reviewer: Box<Account<'info, OrbitMarketAccount>>,

    #[account(
        address = reviewer.wallet
    )]
    pub wallet: Signer<'info>,

    /////////////////////////////////////////////////
    /// EXTRANEOUS CPI
    #[account(
        seeds = [b"market_authority"],
        bump
    )]
    pub commission_auth: SystemAccount<'info>,
    
    pub commission_program: Program<'info, OrbitCommissionMarket>,

    pub accounts_program: Program<'info, OrbitMarketAccounts>
}

impl <'a> OrbitMarketAccountTrait<'a, LeaveReview<'a>> for CommissionTransaction{
    fn leave_review(ctx: Context<LeaveReview>, rating: u8) -> Result<()>{
        if ctx.accounts.reviewer.key() == ctx.accounts.reviewed_account.key(){
            return err!(ReviewErrors::InvalidReviewAuthority)
        };
        if rating == 0 || rating > 5{
            return err!(ReviewErrors::RatingOutsideRange)
        };

        if ctx.accounts.commission_transaction.metadata.seller == ctx.accounts.reviewer.voter_id && !ctx.accounts.commission_transaction.metadata.reviews.seller{
            if let Some(auth_bump) = ctx.bumps.get("commission_auth"){
                orbit_transaction::submit_rating_with_signer!(
                    ctx.accounts.accounts_program.to_account_info(),
                    ctx.accounts.reviewed_account.to_account_info(),
                    ctx.accounts.commission_auth.to_account_info(),
                    ctx.accounts.commission_program.to_account_info(),
                    &[&[b"market_authority", &[*auth_bump]]],
                    rating
                )?;
                ctx.accounts.commission_transaction.metadata.reviews.seller = true;
            }else{
                return err!(MarketAccountErrors::CannotCallOrbitAccountsProgram)
            };
        }else
        if ctx.accounts.commission_transaction.metadata.buyer == ctx.accounts.reviewer.voter_id && !ctx.accounts.commission_transaction.metadata.reviews.buyer{
            if let Some(auth_bump) = ctx.bumps.get("commission_auth"){
                orbit_transaction::submit_rating_with_signer!(
                    ctx.accounts.accounts_program.to_account_info(),
                    ctx.accounts.reviewed_account.to_account_info(),
                    ctx.accounts.commission_auth.to_account_info(),
                    ctx.accounts.commission_program.to_account_info(),
                    &[&[b"market_authority", &[*auth_bump]]],
                    rating
                )?;
                ctx.accounts.commission_transaction.metadata.reviews.buyer = true;
                 
            }else{
                return err!(MarketAccountErrors::CannotCallOrbitAccountsProgram)
            }
        }else
        {
            return err!(ReviewErrors::InvalidReviewAuthority)
        };

        Ok(())
    }

}

//////////////////////////////////////////////////////////////////////////
/// COMMISSION SPECIFIC FIELDS

/// COMMIT PREVIEW FROM SELLER

#[derive(Accounts)]
pub struct CommitPreview<'info>{
    #[account(
        constraint = commission_transaction.metadata.transaction_state == TransactionState::BuyerFunded
    )]
    pub commission_transaction: Box<Account<'info, CommissionTransaction>>,

    #[account(
        constraint = seller_market_account.voter_id == commission_transaction.metadata.seller
    )]
    pub seller_market_account: Account<'info, OrbitMarketAccount>,

    #[account(
        seeds = [
            b"seller_transactions",
            (&(orbit_transaction::TransactionType::Commissions).try_to_vec()?).as_slice(),
            &seller_market_account.voter_id.to_le_bytes()
        ], 
        bump,
        seeds::program = &orbit_transaction::id()
    )]
    pub seller_transactions: Box<Account<'info, SellerOpenTransactions>>,

    #[account(
        address = seller_market_account.wallet
    )]
    pub seller_wallet: Signer<'info>,
}

pub fn commit_preview_handler(ctx: Context<CommitPreview>, link: String) -> Result<()>{
    ctx.accounts.commission_transaction.preview_address = link;
    Ok(())
}

/// RATE UPDATE UTILS FOR PREVIEW

#[derive(Accounts)]
pub struct UpdateRate<'info>{
    #[account(
        mut,
        constraint = commission_transaction.metadata.transaction_state == TransactionState::BuyerFunded
    )]
    pub commission_transaction: Box<Account<'info, CommissionTransaction>>,

    #[account(
        has_one = wallet,
        constraint = 
        (proposer_account.voter_id == commission_transaction.metadata.seller) ||
        (proposer_account.voter_id == commission_transaction.metadata.buyer)
    )]
    pub proposer_account: Account<'info, OrbitMarketAccount>,

    pub wallet: Signer<'info>,
}

pub fn propose_rate_handler(ctx: Context<UpdateRate>, new_rate: u8) -> Result<()>{
    ctx.accounts.commission_transaction.preview_rate = new_rate;
    ctx.accounts.commission_transaction.last_rate_offerer = ctx.accounts.proposer_account.voter_id;
    Ok(())
}

pub fn accept_rate_handler(ctx: Context<UpdateRate>) -> Result<()>{
    if ctx.accounts.proposer_account.voter_id == ctx.accounts.commission_transaction.last_rate_offerer{
        return err!(CommissionMarketErrors::InvalidRateAcceptor)
    };
    ctx.accounts.commission_transaction.last_rate_offerer = ctx.accounts.commission_transaction.metadata.seller;
    ctx.accounts.commission_transaction.close_rate = ctx.accounts.commission_transaction.preview_rate;
    Ok(())
}
