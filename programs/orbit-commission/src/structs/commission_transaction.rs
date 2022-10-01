use anchor_lang::prelude::*;
use transaction::transaction_struct::OrbitTransaction;

#[account]
pub struct CommissionTransaction{
    pub metadata: OrbitTransaction,

    pub preview_addr: String,
    pub preview_rate: u8,
    pub last_rate_offerer: Pubkey,

    pub close_rate: u8,

    pub data_address: String, // 64
    pub num_keys: u64, // 8
    pub key_arr: Vec<Pubkey>, // up to 2048
    pub final_decision: BuyerDecisionState, // 1
}