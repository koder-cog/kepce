use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use chrono::{NaiveDate, Utc};
use uuid::Uuid;
use sea_orm::{
    DatabaseConnection, EntityTrait, ColumnTrait, QueryFilter, Set,
    sea_query::{OnConflict, Expr}
};
use shared::entities::{api_usage_logs, prelude::*};

#[derive(Debug)]
pub struct UsageTracker {
    inner: Arc<Mutex<UsageTrackerInner>>,
}

#[derive(Debug, Default)]
struct UsageTrackerInner {
    // api_key_id -> (requests_today, date)
    cache: HashMap<Uuid, (i32, NaiveDate)>,
    // api_key_id -> (requests_to_add, errors_to_add)
    buffer: HashMap<Uuid, (i32, i32)>,
}

impl UsageTracker {
    pub fn new(db: DatabaseConnection) -> Self {
        let tracker = Self {
            inner: Arc::new(Mutex::new(UsageTrackerInner::default())),
        };
        
        let tracker_clone = tracker.inner.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(10));
            loop {
                interval.tick().await;
                if let Err(e) = Self::flush_buffer(&db, &tracker_clone).await {
                    tracing::error!("Failed to flush API usage logs buffer: {:?}", e);
                }
            }
        });

        tracker
    }

    pub async fn record_request(
        &self,
        db: &DatabaseConnection,
        api_key_id: Uuid,
        tier: &str,
        is_error: bool,
    ) -> Result<(), String> {
        let today = Utc::now().date_naive();
        let limit = Self::get_tier_limit(tier);

        let mut inner = self.inner.lock().await;

        let mut current_requests = 0;
        let mut needs_db_lookup = false;

        if let Some((cached_requests, cached_date)) = inner.cache.get(&api_key_id) {
            if *cached_date == today {
                current_requests = *cached_requests;
            } else {
                needs_db_lookup = true;
            }
        } else {
            needs_db_lookup = true;
        }

        if needs_db_lookup {
            let db_log = ApiUsageLogs::find()
                .filter(api_usage_logs::Column::ApiKeyId.eq(api_key_id))
                .filter(api_usage_logs::Column::Date.eq(today))
                .one(db)
                .await
                .map_err(|e| format!("Database error: {:?}", e))?;

            current_requests = db_log.map(|l| l.requests).unwrap_or(0);
            inner.cache.insert(api_key_id, (current_requests, today));
        }

        if current_requests >= limit {
            return Err(format!(
                "Günlük API kullanım sınırına ulaştınız (Limit: {}/gün).",
                limit
            ));
        }

        let new_count = current_requests + 1;
        inner.cache.insert(api_key_id, (new_count, today));

        let entry = inner.buffer.entry(api_key_id).or_insert((0, 0));
        entry.0 += 1;
        if is_error {
            entry.1 += 1;
        }

        Ok(())
    }

    fn get_tier_limit(tier: &str) -> i32 {
        match tier {
            "ticari" => 100_000,
            _ => 2500, // free / default
        }
    }

    async fn flush_buffer(
        db: &DatabaseConnection,
        tracker: &Mutex<UsageTrackerInner>,
    ) -> Result<(), sea_orm::DbErr> {
        let buffer_to_flush = {
            let mut inner = tracker.lock().await;
            std::mem::take(&mut inner.buffer)
        };

        if buffer_to_flush.is_empty() {
            return Ok(());
        }

        let today = Utc::now().date_naive();

        for (api_key_id, (reqs, errs)) in buffer_to_flush {
            let active_model = api_usage_logs::ActiveModel {
                api_key_id: Set(api_key_id),
                date: Set(today),
                requests: Set(reqs),
                errors: Set(errs),
                ..Default::default()
            };

            let query = ApiUsageLogs::insert(active_model)
                .on_conflict(
                    OnConflict::columns([
                        api_usage_logs::Column::ApiKeyId,
                        api_usage_logs::Column::Date,
                    ])
                    .value(
                        api_usage_logs::Column::Requests,
                        Expr::col(api_usage_logs::Column::Requests).add(reqs),
                    )
                    .value(
                        api_usage_logs::Column::Errors,
                        Expr::col(api_usage_logs::Column::Errors).add(errs),
                    )
                    .to_owned()
                );

            query.exec(db).await?;
        }

        Ok(())
    }
}
