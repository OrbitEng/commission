use anchor_lang::prelude::*;
use product::product_struct::OrbitProduct;

// disc ee26edf0 [238, 38, 237, 240, 132, 29, 235, 255]
#[account]
pub struct CommissionProduct{
    pub metadata: OrbitProduct
}