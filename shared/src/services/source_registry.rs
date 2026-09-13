//! Kaynak Bilgi Tabanı ve Soy Kütüğü (Source Registry & Lineage).
//!
//! Üçüncü taraf kazıma kaynaklarının güven seviyelerini, ait oldukları
//! soy/ekosistem ailelerini (`SourceFamily`) ve çapraz doğrulama kurallarını yönetir.

use serde::{Deserialize, Serialize};

/// Kaynağın ait olduğu geliştirici veya altyapı ekosistemi.
///
/// Aynı aileye mensup iki kaynak birbirinden bağımsız kabul edilemez (klon / döngüsel doğrulama engeli).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SourceFamily {
    /// Fiziksel pano fotoğrafı veya yetkili saha teyidi.
    Field,
    /// kykyemek.com ve ilişkili toplayıcılar.
    Kykyemek,
    /// kykmenum.com ve ilişkili toplayıcılar.
    Kykmenum,
    /// yurtmenu.net, kykyemekliste.com, alidnmz05 GitHub deposu ve türevleri.
    Yurtmenu,
    /// Bilinmeyen veya tanımlanamayan harici kaynaklar.
    Unknown,
}

/// Kaynağın güven katmanı ve sisteme kabul edilme politikası.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TrustTier {
    /// Karantinada olan veya uydurma veri geçmişi bulunan kaynaklar.
    /// Tek başınayken onaylanamaz, doğrudan 'pending' olarak saklanır.
    Quarantined = 1,
    /// Doğrulanmamış veya güvenilmeyen serbest kaynaklar.
    Untrusted = 2,
    /// Bağımsız üçüncü taraf toplayıcılar.
    Aggregator = 3,
    /// Saha teyitli, fiziksel pano veya yönetici girdisi.
    GroundTruth = 4,
}

/// Çözümlenen kaynak meta verisi.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceMetadata {
    pub source_type: String,
    pub family: SourceFamily,
    pub tier: TrustTier,
    pub priority: i32,
    pub is_auto_approvable: bool,
}

/// Kaynak çözümleme ve konsensüs denetleyicisi.
pub struct SourceRegistry;

impl SourceRegistry {
    /// Verilen kaynak metnini (`source_type`) analiz ederek kimliğini,
    /// ailesini ve öncelik katsayısını belirler.
    pub fn resolve(source_type: &str) -> SourceMetadata {
        let s = source_type.to_lowercase();

        // 1. Saha & Yönetici (Ground Truth - Tier 1)
        if s.starts_with("kepce-admin") {
            return SourceMetadata {
                source_type: source_type.to_string(),
                family: SourceFamily::Field,
                tier: TrustTier::GroundTruth,
                priority: 10,
                is_auto_approvable: true,
            };
        }
        if s.starts_with("kepce-kullanici") || s.contains("pano") {
            return SourceMetadata {
                source_type: source_type.to_string(),
                family: SourceFamily::Field,
                tier: TrustTier::GroundTruth,
                priority: 8,
                is_auto_approvable: true,
            };
        }

        // 2. Yurtmenu Ailesi (Karantina / Klon Ekosistemi - Tier 3)
        // yurtmenu.net ve kykyemekliste.com aynı altyapıyı paylaşır.
        if s.contains("yurtmenu") || s.contains("kykyemekliste") || s.contains("alidnmz05") {
            return SourceMetadata {
                source_type: source_type.to_string(),
                family: SourceFamily::Yurtmenu,
                tier: TrustTier::Quarantined,
                priority: 5,
                is_auto_approvable: false,
            };
        }

        // 3. Kykyemek Ailesi (Bağımsız Toplayıcı - Tier 2)
        if s.contains("kykyemek") {
            return SourceMetadata {
                source_type: source_type.to_string(),
                family: SourceFamily::Kykyemek,
                tier: TrustTier::Aggregator,
                priority: 6,
                is_auto_approvable: true,
            };
        }

        // 4. Kykmenum Ailesi (Bağımsız Toplayıcı - Tier 2)
        if s.contains("kykmenum") {
            return SourceMetadata {
                source_type: source_type.to_string(),
                family: SourceFamily::Kykmenum,
                tier: TrustTier::Aggregator,
                priority: 4,
                is_auto_approvable: true,
            };
        }

        // 5. Diğer üçüncü taraf toplayıcılar (kykmenu.com.tr vb.)
        if s.contains("kykmenu") {
            return SourceMetadata {
                source_type: source_type.to_string(),
                family: SourceFamily::Unknown,
                tier: TrustTier::Aggregator,
                priority: 3,
                is_auto_approvable: false,
            };
        }

        // 6. Bilinmeyen veya tanımlanamayan kaynaklar
        SourceMetadata {
            source_type: source_type.to_string(),
            family: SourceFamily::Unknown,
            tier: TrustTier::Untrusted,
            priority: 1,
            is_auto_approvable: false,
        }
    }

    /// Verilen kaynağın karantinada olup olmadığını döner.
    pub fn is_quarantined(source_type: &str) -> bool {
        Self::resolve(source_type).tier == TrustTier::Quarantined
    }

    /// İki farklı kaynağın bağımsız bir konsensüs oluşturup oluşturamayacağını kontrol eder.
    ///
    /// Aynı aileden gelen iki kaynak (örneğin `yurtmenu.net` ve `kykyemekliste.com`)
    /// aynı veri tabanını veya geliştiriciyi paylaştığı için asla bağımsız konsensüs oluşturamaz.
    pub fn is_cross_family_consensus(source_a: &str, source_b: &str) -> bool {
        let meta_a = Self::resolve(source_a);
        let meta_b = Self::resolve(source_b);

        // İki kaynak aynı aileye aitse bağımsız konsensüs sağlanamaz.
        if meta_a.family == meta_b.family {
            return false;
        }

        // Güvenilmeyen veya bilinmeyen kaynak konsensüs temeli olamaz.
        if meta_a.tier == TrustTier::Untrusted || meta_b.tier == TrustTier::Untrusted {
            return false;
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_family_resolution() {
        let admin = SourceRegistry::resolve("kepce-admin");
        assert_eq!(admin.family, SourceFamily::Field);
        assert_eq!(admin.tier, TrustTier::GroundTruth);
        assert_eq!(admin.priority, 10);
        assert!(admin.is_auto_approvable);

        let kykyemek = SourceRegistry::resolve("kykyemek.com");
        assert_eq!(kykyemek.family, SourceFamily::Kykyemek);
        assert_eq!(kykyemek.tier, TrustTier::Aggregator);
        assert_eq!(kykyemek.priority, 6);
        assert!(kykyemek.is_auto_approvable);

        let yurtmenu = SourceRegistry::resolve("yurtmenu.net");
        assert_eq!(yurtmenu.family, SourceFamily::Yurtmenu);
        assert_eq!(yurtmenu.tier, TrustTier::Quarantined);
        assert_eq!(yurtmenu.priority, 5);
        assert!(!yurtmenu.is_auto_approvable);

        let kykyemekliste = SourceRegistry::resolve("kykyemekliste.com");
        assert_eq!(kykyemekliste.family, SourceFamily::Yurtmenu);
        assert_eq!(kykyemekliste.tier, TrustTier::Quarantined);
        assert!(!kykyemekliste.is_auto_approvable);

        let kykmenum = SourceRegistry::resolve("kykmenum.com");
        assert_eq!(kykmenum.family, SourceFamily::Kykmenum);
        assert_eq!(kykmenum.priority, 4);

        let unknown = SourceRegistry::resolve("random-bot");
        assert_eq!(unknown.family, SourceFamily::Unknown);
        assert_eq!(unknown.tier, TrustTier::Untrusted);
    }

    #[test]
    fn test_cross_family_consensus() {
        // yurtmenu ve kykyemekliste aynı ailede (Yurtmenu). Konsensüs oluşamaz.
        assert!(!SourceRegistry::is_cross_family_consensus(
            "yurtmenu.net",
            "kykyemekliste.com"
        ));

        // yurtmenu ve kykyemek farklı ailelerde. Konsensüs geçerlidir.
        assert!(SourceRegistry::is_cross_family_consensus(
            "yurtmenu.net",
            "kykyemek.com"
        ));

        // yurtmenu ve saha panosu farklı ailelerde. Konsensüs geçerlidir.
        assert!(SourceRegistry::is_cross_family_consensus(
            "yurtmenu.net",
            "kepce-admin"
        ));

        // Bilinmeyen kaynakla konsensüs kurulamaz.
        assert!(!SourceRegistry::is_cross_family_consensus(
            "yurtmenu.net",
            "random-untrusted-scraper"
        ));
    }
}
