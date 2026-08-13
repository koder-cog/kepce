// Kepçe Shared — Servisler
// =========================
//
// Veritabanı seviyesinde paylaşılan iş mantığı.
//
// immutable_store.rs → Hash Chain işlemleri
//   - compute_menu_hash()   → SHA-256 hash hesaplama
//   - get_previous_hash()   → Önceki menünün hash'ini getirme
//   - write_menu_hash()     → Menü hash'ini hesaplayıp yazma
//

pub mod alerting;
pub mod content_guard;
pub mod immutable_store;
