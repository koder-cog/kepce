pub mod domain;
pub mod entities;
pub mod infra;

// Geriye dönük uyumluluk (api ve worker içindeki eski shared::services::* importlarının kırılmaması için):
pub mod services {
    pub use crate::domain::calendar;
    pub use crate::domain::menu::categorizer;
    pub use crate::domain::quality::content_guard;
    pub use crate::domain::quality::cqs_score as quality_score;
    pub use crate::domain::quality::normalizer;
    pub use crate::infra::alerting;
    pub use crate::infra::immutable_store;
    pub use crate::infra::source_registry;
}
