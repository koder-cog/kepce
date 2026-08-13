// Kepçe Shared Crate — Ortak Kütüphane
// ======================================
//
// api ve worker crate'leri tarafından paylaşılan kod.
// İçerik:
//   - entities/   → SeaORM veritabanı modelleri (mevcut yapıdan taşınacak)
//   - services/   → Veritabanı seviyesinde paylaşılan iş mantığı
//
// Bu crate'in amacı:
//   API ve Worker aynı veritabanı modellerini kullanır.
//   Aynı dish_matcher, immutable_store ve content_guard fonksiyonlarını kullanır.
// Bu kodu iki yere kopyalamak yerine, shared crate olarak paylaşırız.

pub mod entities;
pub mod services;
