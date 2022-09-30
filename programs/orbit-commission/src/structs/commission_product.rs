use anchor_lang::prelude::*;
use product::product_struct::OrbitProduct;

#[account]
pub struct CommissionProduct{
    pub metadata: OrbitProduct,
}