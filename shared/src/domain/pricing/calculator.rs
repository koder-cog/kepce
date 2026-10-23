//! Fiyat ve öğün maliyet hesaplayıcı
use sea_orm::prelude::Decimal;

pub fn calculate_total_price(items: &[Decimal]) -> Decimal {
    items.iter().copied().sum()
}
