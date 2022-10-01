use anchor_lang::{
    prelude::*,
    AccountsClose,
    solana_program::{
        program::invoke,
        system_instruction::transfer
    }
};
use transaction::{
    transaction_trait::OrbitTransactionTrait,
    transaction_struct::TransactionState,
    TransactionReviews,
    ReviewErrors,
    transaction_utils::*
};
use market_accounts::{
    market_account::OrbitMarketAccount, program::OrbitMarketAccounts,
    structs::market_account_trait::OrbitMarketAccountTrait,
    MarketAccountErrors
};
use anchor_spl::token::accessor::amount;
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
    BuyerDecisionState, program::OrbitCommissionMarket
};

////////////////////////////////////////////////////////////////////
/// ORBIT BASE TRANSACTION FUNCTIONALITIES

#[derive(Accounts)]
pub struct CloseTransactionAccount<'info>{
    #[account(
        mut,
        constraint = commission_transaction.metadata.transaction_state == TransactionState::Closed,
    )]
    pub commission_transaction: Box<Account<'info, CommissionTransaction>>,

    #[account(
        constraint = 
            (market_account.key() == commission_transaction.metadata.buyer) ||
            (market_account.key() == commission_transaction.metadata.seller),

        seeds = [
            b"orbit_account",
            wallet.key().as_ref()
        ],
        bump,
        seeds::program = market_accounts::ID
    )]
    pub market_account: Account<'info, OrbitMarketAccount>,

    #[account(
        address = market_account.wallet
    )]
    pub wallet: Signer<'info>,

    #[account(
        mut,
        address = market_account.wallet
    )]
    pub buyer_wallet: SystemAccount<'info>
}

impl<'a, 'b, 'c, 'd, 'e, 'f, 'g> OrbitTransactionTrait<'a, 'b, 'c, 'd, 'e, 'f, 'g, OpenCommissionTransactionSol<'a>, OpenCommissionTransactionSpl<'b>, CloseCommissionTransactionSol<'c>, CloseCommissionTransactionSpl<'d>, FundEscrowSol<'e>, FundEscrowSpl<'f>, CloseTransactionAccount<'g>> for CommissionTransaction{
    fn open_sol(ctx: Context<OpenCommissionTransactionSol>, mut price: u64, use_discount: bool) -> Result<()>{
        if use_discount && ctx.accounts.buyer_account.dispute_discounts > 0{
            ctx.accounts.commission_transaction.metadata.rate = 100;
            price = price * 95 / 100;
            ctx.accounts.buyer_account.dispute_discounts -=1;
        }else{
            ctx.accounts.commission_transaction.metadata.rate = 95
        }
        ctx.accounts.commission_transaction.metadata.buyer = ctx.accounts.buyer_account.key();
        ctx.accounts.commission_transaction.metadata.seller = ctx.accounts.commission_product.metadata.seller;
        ctx.accounts.commission_transaction.metadata.product = ctx.accounts.commission_product.key();
        ctx.accounts.commission_transaction.metadata.transaction_state = TransactionState::Opened;
        ctx.accounts.commission_transaction.metadata.transaction_price = price;
        ctx.accounts.commission_transaction.metadata.funded = false;
        ctx.accounts.commission_transaction.metadata.currency = ctx.accounts.commission_product.metadata.currency;

        ctx.accounts.commission_transaction.num_keys = 0;

        ctx.accounts.commission_transaction.metadata.escrow_account = ctx.accounts.escrow_account.key();
        ctx.accounts.commission_transaction.final_decision = BuyerDecisionState::Null;

        ctx.accounts.commission_transaction.metadata.reviews = TransactionReviews{
            buyer: false,
            seller: false
        };

        Ok(())
    }

    fn open_spl(ctx: Context<OpenCommissionTransactionSpl>, mut price: u64, use_discount: bool) -> Result<()>{
        if use_discount && ctx.accounts.buyer_account.dispute_discounts > 0{
            ctx.accounts.commission_transaction.metadata.rate = 100;
            price = price * 95 / 100;
            ctx.accounts.buyer_account.dispute_discounts -= 1;
        }else{
            ctx.accounts.commission_transaction.metadata.rate = 95
        }
        ctx.accounts.commission_transaction.metadata.buyer = ctx.accounts.buyer_account.key();
        ctx.accounts.commission_transaction.metadata.seller = ctx.accounts.commission_product.metadata.seller;
        ctx.accounts.commission_transaction.metadata.product = ctx.accounts.commission_product.key();
        ctx.accounts.commission_transaction.metadata.transaction_state = TransactionState::Opened;
        ctx.accounts.commission_transaction.metadata.transaction_price = price;
        ctx.accounts.commission_transaction.metadata.funded = false;
        ctx.accounts.commission_transaction.metadata.currency = ctx.accounts.commission_product.metadata.currency;
        
        ctx.accounts.commission_transaction.num_keys = 0;

        ctx.accounts.commission_transaction.metadata.escrow_account = ctx.accounts.escrow_account.key();
        ctx.accounts.commission_transaction.final_decision = BuyerDecisionState::Null;

        ctx.accounts.commission_transaction.metadata.reviews = TransactionReviews{
            buyer: false,
            seller: false
        };
        Ok(())
    }

    fn close_sol(ctx: Context<'_, '_, '_, 'c, CloseCommissionTransactionSol<'c>>) -> Result<()>{
        match ctx.bumps.get("escrow_account"){
            Some(escrow_bump) => {
                if (ctx.accounts.commission_transaction.close_rate == 95)
                && (ctx.accounts.commission_transaction.final_decision == BuyerDecisionState::Accept){
                    let bal = ctx.accounts.escrow_account.lamports();
                    let mut residual_amt = bal * 5/100;
                    if  (ctx.accounts.buyer_account.reflink != Pubkey::new(&[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0])) &&
                        (ctx.remaining_accounts[0].key() == ctx.accounts.buyer_account.reflink)
                    {
                        let reflink_amt = bal * 25 / 10000;
                        residual_amt = bal * 45/1000;
                        close_escrow_sol_flat(
                            ctx.accounts.escrow_account.to_account_info(),
                            ctx.accounts.buyer_wallet.to_account_info(),
                            &[&[b"orbit_escrow_account", ctx.accounts.commission_transaction.key().as_ref(), &[*escrow_bump]]],
                            reflink_amt
                        ).expect("couldnt close escrow");
                        match remaining_accounts_to_wallet(ctx.remaining_accounts){
                            Ok(reflink_wallet) => {
                                close_escrow_sol_flat(
                                    ctx.accounts.escrow_account.to_account_info(),
                                    reflink_wallet.to_account_info(),
                                    &[&[b"orbit_escrow_account", ctx.accounts.commission_transaction.key().as_ref(), &[*escrow_bump]]],
                                    reflink_amt
                                ).expect("couldnt close escrow");
                                reflink_wallet.exit(ctx.program_id)?;
                            },
                            Err(e) => return Err(e)
                        }
                    }
                    close_escrow_sol_flat(
                        ctx.accounts.escrow_account.to_account_info(),
                        ctx.accounts.multisig_wallet.to_account_info(),
                        &[&[b"orbit_escrow_account", ctx.accounts.commission_transaction.key().as_ref(), &[*escrow_bump]]],
                        residual_amt
                    ).expect("couldnt close escrow");
                }

                close_escrow_sol_rate(
                    ctx.accounts.escrow_account.to_account_info(),
                    ctx.accounts.seller_wallet.to_account_info(),
                    &[&[b"orbit_escrow_account", ctx.accounts.commission_transaction.key().as_ref(), &[*escrow_bump]]],
                    ctx.accounts.commission_transaction.close_rate
                ).expect("could not transfer tokens");
                close_escrow_sol_rate(
                    ctx.accounts.escrow_account.to_account_info(),
                    ctx.accounts.buyer_wallet.to_account_info(),
                    &[&[b"orbit_escrow_account", ctx.accounts.commission_transaction.key().as_ref(), &[*escrow_bump]]],
                    100
                ).expect("could not transfer tokens");
            },
            None => return err!(CommissionMarketErrors::InvalidEscrowBump)
        };

        match ctx.bumps.get("commission_auth"){
            Some(auth_bump) => post_tx_incrementing(
                ctx.accounts.market_account_program.to_account_info(),
                ctx.accounts.buyer_account.to_account_info(),
                ctx.accounts.seller_account.to_account_info(),
                ctx.accounts.commission_auth.to_account_info(),
                ctx.accounts.commission_program.to_account_info(),
                &[&[b"market_authority", &[*auth_bump]]]
            ),
            None => return err!(CommissionMarketErrors::InvalidAuthBump)
        }.expect("could not properly invoke market-accounts program");

        ctx.accounts.commission_transaction.metadata.transaction_state = TransactionState::Closed;
        Ok(())
    }

    fn close_spl(ctx: Context<'_, '_, '_, 'd, CloseCommissionTransactionSpl<'d>>) -> Result<()>{
        match ctx.bumps.get("commission_auth"){
            Some(auth_bump) => {
                if (ctx.accounts.commission_transaction.close_rate == 95)
                && (ctx.accounts.commission_transaction.final_decision == BuyerDecisionState::Accept){
                    let bal = amount(&ctx.accounts.escrow_account.to_account_info()).expect("could not deserialize token account");
                    let mut residual_amt = bal * 5/100;
                    if  (ctx.accounts.buyer_account.reflink != Pubkey::new(&[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0])) &&
                        (ctx.remaining_accounts[0].key() == ctx.accounts.buyer_account.reflink)
                    {
                        let reflink_amt = bal * 25 / 10000;
                        residual_amt = bal * 45/1000;
                        close_escrow_spl_flat(
                            ctx.accounts.token_program.to_account_info(),
                            ctx.accounts.escrow_account.to_account_info(),
                            ctx.accounts.buyer_token_account.to_account_info(),
                            ctx.accounts.commission_auth.to_account_info(),
                            &[&[b"market_authority", &[*auth_bump]]],
                            reflink_amt
                        ).expect("couldnt close escrow");
                        

                        match remaining_accounts_to_token_account(ctx.remaining_accounts){
                            Ok(reflink_token_account) => {
                                close_escrow_spl_flat(
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
                    close_escrow_spl_flat(
                        ctx.accounts.token_program.to_account_info(),
                        ctx.accounts.escrow_account.to_account_info(),
                        ctx.accounts.multisig_ata.to_account_info(),
                        ctx.accounts.commission_auth.to_account_info(),
                        &[&[b"market_authority", &[*auth_bump]]],
                        residual_amt
                    ).expect("couldnt close escrow");
                }

                close_escrow_spl_rate(
                    ctx.accounts.token_program.to_account_info(),
                    ctx.accounts.escrow_account.to_account_info(),
                    ctx.accounts.seller_token_account.to_account_info(),
                    ctx.accounts.commission_auth.to_account_info(),
                    &[&[b"market_authority", &[*auth_bump]]],
                    ctx.accounts.commission_transaction.metadata.transaction_price,
                    ctx.accounts.commission_transaction.close_rate
                ).expect("could not transfer tokens");
                close_escrow_spl_rate(
                    ctx.accounts.token_program.to_account_info(),
                    ctx.accounts.escrow_account.to_account_info(),
                    ctx.accounts.buyer_token_account.to_account_info(),
                    ctx.accounts.commission_auth.to_account_info(),
                    &[&[b"market_authority", &[*auth_bump]]],
                    ctx.accounts.commission_transaction.metadata.transaction_price,
                    100
                ).expect("could not transfer tokens");
                post_tx_incrementing(
                    ctx.accounts.market_account_program.to_account_info(),
                    ctx.accounts.buyer_account.to_account_info(),
                    ctx.accounts.seller_account.to_account_info(),
                    ctx.accounts.commission_auth.to_account_info(),
                    ctx.accounts.commission_program.to_account_info(),
                    &[&[b"market_authority", &[*auth_bump]]]
                )
            },
            None => return err!(CommissionMarketErrors::InvalidAuthBump)
        }.expect("could not properly invoke market-accounts program");

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
                    from: ctx.accounts.buyer_spl_wallet.to_account_info(),
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
        ctx.accounts.commission_transaction.close(ctx.accounts.market_account.to_account_info())
    }
}

//////////////////////////////////////////////////////////////////////////
/// BUYER CONFIRMATIONS

#[derive(Accounts)]
pub struct BuyerConfirmation<'info>{
    #[account(
        mut,
        constraint = commission_transaction.metadata.buyer == buyer_account.key(),
        constraint = commission_transaction.final_decision == BuyerDecisionState::Null,
    )]
    pub commission_transaction: Box<Account<'info, CommissionTransaction>>,

    #[account(
        mut,
        seeds = [
            b"orbit_account",
            wallet.key().as_ref()
        ],
        bump,
        seeds::program = market_accounts::ID
    )]
    pub buyer_account: Account<'info, OrbitMarketAccount>,

    #[account(
        address = buyer_account.wallet
    )]
    pub wallet: Signer<'info>,
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

pub fn deny_accept_handler(ctx: Context<BuyerConfirmation>) -> Result<()>{
    if ctx.accounts.commission_transaction.metadata.transaction_state != TransactionState::BuyerConfirmedDelivery{
        return err!(CommissionMarketErrors::DidNotConfirmDelivery);
    }
    if ctx.accounts.commission_transaction.metadata.rate == 100{
        ctx.accounts.buyer_account.dispute_discounts += 1;
    }
    ctx.accounts.commission_transaction.metadata.rate = 0;
    ctx.accounts.commission_transaction.close_rate = 0;
    ctx.accounts.commission_transaction.final_decision = BuyerDecisionState::Declined;
    ctx.accounts.commission_transaction.metadata.transaction_state = TransactionState::BuyerConfirmedProduct;

    Ok(())
}


#[derive(Accounts)]
pub struct EarlyDeclineTransaction<'info>{
    #[account(
        mut,
        constraint = commission_transaction.final_decision == BuyerDecisionState::Null
    )]
    pub commission_transaction: Account<'info, CommissionTransaction>,

    #[account(
        address = commission_transaction.metadata.buyer,
        seeds = [
            b"orbit_account",
            wallet.key().as_ref()
        ],
        bump,
        seeds::program = market_accounts::ID
    )]
    pub buyer: Account<'info, OrbitMarketAccount>,

    #[account(
        address = buyer.wallet
    )]
    pub wallet: Signer<'info>
}

pub fn early_decline_handler(ctx: Context<EarlyDeclineTransaction>) -> Result<()>{
    ctx.accounts.commission_transaction.metadata.transaction_state = TransactionState::BuyerConfirmedProduct;
    ctx.accounts.commission_transaction.final_decision = BuyerDecisionState::Declined;
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
        address = commission_transaction.metadata.seller,
        seeds = [
            b"orbit_account",
            wallet.key().as_ref()
        ],
        bump,
        seeds::program = market_accounts::ID
    )]
    pub seller_account: Account<'info, OrbitMarketAccount>,

    #[account(
        address = seller_account.wallet
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
        address = commission_transaction.metadata.seller,
        seeds = [
            b"orbit_account",
            wallet.key().as_ref()
        ],
        bump,
        seeds::program = market_accounts::ID
    )]
    pub seller_account: Account<'info, OrbitMarketAccount>,

    #[account(
        address = seller_account.wallet,
    )]
    pub wallet: Signer<'info>,
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
        address = commission_transaction.metadata.seller,
        seeds = [
            b"orbit_account",
            wallet.key().as_ref()
        ],
        bump,
        seeds::program = market_accounts::ID
    )]
    pub seller_account: Account<'info, OrbitMarketAccount>,

    #[account(
        address = seller_account.wallet
    )]
    pub wallet: Signer<'info>,
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
    #[account(mut)]
    pub commission_transaction: Box<Account<'info, CommissionTransaction>>,

    #[account(
        mut,
        constraint = 
        (reviewer.key() == commission_transaction.metadata.seller) ||
        (reviewer.key() == commission_transaction.metadata.buyer)
    )]
    pub reviewed_account: Account<'info, OrbitMarketAccount>,

    #[account(
        constraint = 
        (reviewer.key() == commission_transaction.metadata.seller) ||
        (reviewer.key() == commission_transaction.metadata.buyer),
        seeds = [
            b"orbit_account",
            wallet.key().as_ref()
        ],
        bump,
        seeds::program = market_accounts::ID
    )]
    pub reviewer: Account<'info, OrbitMarketAccount>,

    #[account(
        address = reviewer.wallet
    )]
    pub wallet: Signer<'info>,

    pub accounts_program: Program<'info, OrbitMarketAccounts>,

    #[account(
        seeds = [b"market_authority"],
        bump
    )]
    pub commission_auth: SystemAccount<'info>,

    pub commission_program: Program<'info, OrbitCommissionMarket>
}

impl <'a> OrbitMarketAccountTrait<'a, LeaveReview<'a>> for CommissionTransaction{
    fn leave_review(ctx: Context<LeaveReview>, rating: u8) -> Result<()>{
        if ctx.accounts.reviewer.key() == ctx.accounts.reviewed_account.key(){
            return err!(ReviewErrors::InvalidReviewAuthority)
        };
        if rating == 0 || rating > 5{
            return err!(ReviewErrors::RatingOutsideRange)
        };

        if ctx.accounts.commission_transaction.metadata.seller == ctx.accounts.reviewer.key() && !ctx.accounts.commission_transaction.metadata.reviews.seller{
            match ctx.bumps.get("commission_auth"){
                Some(auth_bump) => {
                    submit_rating_with_signer(
                        ctx.accounts.accounts_program.to_account_info(),
                        ctx.accounts.reviewed_account.to_account_info(),
                        ctx.accounts.commission_auth.to_account_info(),
                        ctx.accounts.commission_program.to_account_info(),
                        &[&[b"market_authority", &[*auth_bump]]],
                        rating
                    );
                    ctx.accounts.commission_transaction.metadata.reviews.seller = true;
                },
                None => return err!(MarketAccountErrors::CannotCallOrbitAccountsProgram)
            };
        }else
        if ctx.accounts.commission_transaction.metadata.buyer == ctx.accounts.reviewer.key()  && !ctx.accounts.commission_transaction.metadata.reviews.buyer{
            match ctx.bumps.get("commission_auth"){
                Some(auth_bump) => {
                    submit_rating_with_signer(
                        ctx.accounts.accounts_program.to_account_info(),
                        ctx.accounts.reviewed_account.to_account_info(),
                        ctx.accounts.commission_auth.to_account_info(),
                        ctx.accounts.commission_program.to_account_info(),
                        &[&[b"market_authority", &[*auth_bump]]],
                        rating
                    );
                    ctx.accounts.commission_transaction.metadata.reviews.buyer = true;
                },
                None => return err!(MarketAccountErrors::CannotCallOrbitAccountsProgram)
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
        address = commission_transaction.metadata.seller,
        seeds = [
            b"orbit_account",
            seller_wallet.key().as_ref()
        ],
        bump,
        seeds::program = market_accounts::ID
    )]
    pub seller_account: Account<'info, OrbitMarketAccount>,

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
        constraint = (market_account.key() == commission_transaction.metadata.seller) || (market_account.key() == commission_transaction.metadata.buyer),
        seeds = [
            b"orbit_account",
            wallet.key().as_ref()
        ],
        bump,
        seeds::program = market_accounts::ID
    )]
    pub market_account: Account<'info, OrbitMarketAccount>,

    pub wallet: Signer<'info>,
}

pub fn propose_rate_handler(ctx: Context<UpdateRate>, new_rate: u8) -> Result<()>{
    ctx.accounts.commission_transaction.preview_rate = new_rate;
    ctx.accounts.commission_transaction.last_rate_offerer = ctx.accounts.market_account.key();
    Ok(())
}

pub fn accept_rate_handler(ctx: Context<UpdateRate>) -> Result<()>{
    if ctx.accounts.market_account.key() == ctx.accounts.commission_transaction.last_rate_offerer{
        return err!(CommissionMarketErrors::InvalidRateAcceptor)
    };
    ctx.accounts.commission_transaction.last_rate_offerer = ctx.accounts.commission_transaction.metadata.seller;
    ctx.accounts.commission_transaction.close_rate = ctx.accounts.commission_transaction.preview_rate;
    Ok(())
}
