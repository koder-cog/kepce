pub mod domain;
pub mod jobs;
pub mod pipeline;
pub mod quality;
pub mod sources;
pub mod telemetry;

// Geriye dönük uyumluluk (backward compatibility) katmanı
pub mod parser {
    pub use crate::domain::models;
    pub use crate::pipeline as core;
    pub use crate::pipeline::l0_router::csv_dp as csv_grid;
    pub use crate::pipeline::l0_router::excel;
    pub use crate::pipeline::l0_router::json_dp as json;
    pub use crate::pipeline::l1_vision::orientation;
    pub use crate::pipeline::l2_vlm as llm;
    pub use crate::quality::anomaly;
    pub use crate::quality::cqs_validator as validation;
    pub use crate::quality::decision;
    pub use crate::quality::dictionary;
    pub use crate::quality::normalizer;
    pub use crate::quality::stale_detector;
    pub use crate::quality::takeaway;
    pub use crate::sources::kykmenum;
    pub use crate::sources::kykyemek;
    pub use crate::sources::yurtmenu;
    pub use crate::telemetry::cost_monitor as openrouter_monitor;
    pub use crate::telemetry::profiler;

    pub use crate::pipeline::save_menu_database;
}

pub mod tasks {
    pub use crate::jobs::curation::cleanup;
    pub use crate::jobs::curation::dish_reconciler;
    pub use crate::jobs::curation::enrich_takeaway;
    pub use crate::jobs::curation::quarantine;
    pub use crate::jobs::curation::sanitizer;
    pub use crate::jobs::engagement::comment_gen as comment_generator;
    pub use crate::jobs::engagement::indexnow;
    pub use crate::jobs::engagement::meal_notifier;
    pub use crate::jobs::engagement::telegram_bot;
    pub use crate::jobs::ingestion::backup_export;
    pub use crate::jobs::ingestion::backup_ingest;
    pub use crate::jobs::ingestion::file_ingest;
    pub use crate::jobs::ingestion::historical as historical_ingest;
    pub use crate::jobs::ingestion::fallback_scraper;
    pub use crate::jobs::ingestion::scraper;
}
