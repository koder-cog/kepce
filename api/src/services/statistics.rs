// Kepçe API — Service: İstatistik Servisi
// =========================================
//
// Tüm istatistik hesaplamaları.
//
// Sorumlulukları:
//   1. En çok / en az beğenilen yemekler
//   2. Trend etiketler
//   3. En beğenilen yorumlar (CommentService.enrich kullanır)
//   4. Son yorumlar
//   5. Moderasyon aktivitesi
//   6. İnsanlık istatistikleri (toplam yorum, kullanıcı sayısı vb.)

use sea_orm::*;
use shared::entities::{prelude::*, reports, comments, dish_tags, sea_orm_active_enums::ReportStatusEnum};
use crate::dto::statistics::{TopDishDto, ModerationStatsDto, TrendingTagDto, HumanityStatsDto, ContributorDto, RecentActionDto};

#[derive(FromQueryResult)]
struct TrendingTagResult {
    name: String,
    count: i64,
    category: String,
}

pub struct StatisticsService;

#[derive(Debug)]
pub enum StatsError {
    DatabaseError(DbErr),
}

impl StatisticsService {
    /// En çok veya en az beğenilen yemeklerin (Leaderboard) istatistiği.
    /// Karmaşık JOIN ve Aggregation işlemleri olduğu için SeaORM'un raw SQL özelliği ile çözüyoruz.
    pub async fn get_dish_leaderboard(
        db: &DatabaseConnection,
        limit: u64,
        is_top: bool, // true: en iyi, false: en kötü
        city_slug: Option<String>,
        timeframe: Option<String>,
    ) -> Result<Vec<TopDishDto>, StatsError> {
        // GÜVENLİK: ORDER BY yönü whitelist ile belirlenir, SQL injection riski yok.
        // LIMIT parametresi bind variable ($1) ile geçirilir — string interpolation YASAK.
        let order_clause = if is_top { "DESC" } else { "ASC" };
        
        // Pagination sınırı: DoS önlemi
        let safe_limit = limit.min(100) as i64;
        
        let mut join_clause = "".to_string();
        let mut conditions = Vec::new();
        let mut values: Vec<sea_orm::Value> = vec![safe_limit.into()];
        let param_index = 2;

        if let Some(slug) = city_slug {
            join_clause = "JOIN menus m ON dv.menu_id = m.id JOIN cities ci ON m.city_id = ci.id".to_string();
            conditions.push(format!("ci.slug = ${}", param_index));
            values.push(slug.into());
        }

        if let Some(tf) = timeframe {
            let interval = match tf.as_str() {
                "daily" => "1 day",
                "weekly" => "7 days",
                "monthly" => "30 days",
                "yearly" => "365 days",
                _ => "",
            };
            if !interval.is_empty() {
                conditions.push(format!("dv.created_at >= NOW() - INTERVAL '{}'", interval));
            }
        }

        let where_clause = if conditions.is_empty() {
            "".to_string()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        let sql = format!(
            r#"
            SELECT 
                d.id as dish_id, 
                d.name, 
                CAST(COUNT(dv.id) AS INTEGER) as total_votes,
                CAST(SUM(CASE WHEN dv.sentiment = 'positive' THEN 1 WHEN dv.sentiment = 'negative' THEN -1 ELSE 0 END) AS INTEGER) as score,
                CAST(SUM(CASE WHEN dv.sentiment = 'positive' THEN 1 ELSE 0 END) * 1.0 / NULLIF(COUNT(dv.id), 0) AS DOUBLE PRECISION) as average_rating
            FROM dishes d
            JOIN dish_votes dv ON dv.dish_id = d.id
            {}
            {}
            GROUP BY d.id, d.name
            ORDER BY score {}
            LIMIT $1
            "#,
            join_clause, where_clause, order_clause
        );

        let query = Statement::from_sql_and_values(
            db.get_database_backend(),
            &sql,
            values
        );

        let results = db
            .query_all(query)
            .await
            .map_err(StatsError::DatabaseError)?;
            
        let mut dtos = Vec::new();
        for row in results {
            let dish_id: i32 = row.try_get("", "dish_id").map_err(StatsError::DatabaseError)?;
            let name: String = row.try_get("", "name").map_err(StatsError::DatabaseError)?;
            let total_votes: i32 = row.try_get("", "total_votes").map_err(StatsError::DatabaseError)?;
            let score: i32 = row.try_get("", "score").map_err(StatsError::DatabaseError)?;
            let average_rating: Option<f64> = row.try_get("", "average_rating").ok();
            dtos.push(TopDishDto { dish_id, name, total_votes, score, average_rating });
        }

        Ok(dtos)
    }

    /// Developer/Admin istatistikleri için moderasyon özetini getirir
    pub async fn get_moderation_stats(
        db: &DatabaseConnection,
    ) -> Result<ModerationStatsDto, StatsError> {
        // Rapor istatistiklerini tek sorguda GROUP BY ile çekiyoruz
        let stats = Reports::find()
            .select_only()
            .column(reports::Column::Status)
            .column_as(reports::Column::Id.count(), "count")
            .group_by(reports::Column::Status)
            .into_tuple::<(ReportStatusEnum, i64)>()
            .all(db)
            .await
            .map_err(StatsError::DatabaseError)?;
            
        let mut pending = 0;
        let mut resolved = 0;
        let mut dismissed = 0;
        
        for (status, count) in stats {
            match status {
                ReportStatusEnum::Pending => pending = count,
                ReportStatusEnum::Resolved => resolved = count,
                ReportStatusEnum::Dismissed => dismissed = count,
            }
        }
            
        let bans = UserBlocks::find()
            .count(db)
            .await
            .map_err(StatsError::DatabaseError)?;
        
        let deleted_comments = Comments::find()
            .filter(comments::Column::IsDeleted.eq(true))
            .count(db)
            .await
            .map_err(StatsError::DatabaseError)? as i64;
            
        let sql = r#"
            SELECT 
                u.username AS nickname, 
                u.opt_out_statistics,
                w.message AS action,
                'warning' as action_type
            FROM user_warnings w
            JOIN users u ON w.user_id = u.id
            ORDER BY w.created_at DESC
            LIMIT 5
        "#;

        let query = Statement::from_sql_and_values(
            db.get_database_backend(),
            sql,
            vec![]
        );

        let results = db
            .query_all(query)
            .await
            .map_err(StatsError::DatabaseError)?;

        let mut recent_actions = Vec::new();
        for row in results {
            let opt_out: bool = row.try_get("", "opt_out_statistics").unwrap_or(false);
            let nickname: String = if opt_out {
                "Anonim".to_string()
            } else {
                row.try_get("", "nickname").unwrap_or_else(|_| "Bilinmeyen".to_string())
            };
            let action: String = row.try_get("", "action").unwrap_or_else(|_| "Bilinmeyen işlem".to_string());
            let action_type: String = row.try_get("", "action_type").unwrap_or_else(|_| "info".to_string());
            
            // Check if action string contains "uzaklaştırıldı" to set it as a ban type for backward compatibility
            // if we are just mocking action_type from 'warning' static string
            let mut final_type = action_type;
            if action.contains("uzaklaştırıldı") {
                final_type = "ban".to_string();
            }

            recent_actions.push(RecentActionDto { nickname, action, action_type: final_type });
        }
        
        Ok(ModerationStatsDto {
            pending_reports_count: pending as i32,
            resolved_reports_count: resolved as i32,
            auto_dismissed_count: dismissed as i32,
            active_bans_count: bans as i32,
            deleted_comments,
            recent_actions,
        })
    }

    pub async fn get_trending_tags(db: &DatabaseConnection, limit: u64) -> Result<Vec<TrendingTagDto>, StatsError> {
        let safe_limit = limit.min(50) as i64;
        let sql = r#"
            SELECT 
                t.name, 
                CAST(COUNT(dt.dish_id) AS BIGINT) as count,
                t.category
            FROM tags t
            JOIN dish_tags dt ON t.id = dt.tag_id
            WHERE dt.created_at >= NOW() - INTERVAL '30 days'
            GROUP BY t.id, t.name, t.category
            ORDER BY count DESC
            LIMIT $1
        "#;
        
        let query = Statement::from_sql_and_values(
            db.get_database_backend(),
            sql,
            vec![safe_limit.into()]
        );
        
        let results = TrendingTagResult::find_by_statement(query).all(db).await
            .map_err(StatsError::DatabaseError)?;
            
        Ok(results.into_iter().map(|r| TrendingTagDto {
            name: r.name,
            count: r.count,
            category: r.category,
        }).collect())
    }

    pub async fn get_humanity_stats(db: &DatabaseConnection) -> Result<HumanityStatsDto, StatsError> {
        let stats = Reports::find()
            .select_only()
            .column(reports::Column::Status)
            .column_as(reports::Column::Id.count(), "count")
            .group_by(reports::Column::Status)
            .into_tuple::<(ReportStatusEnum, i64)>()
            .all(db)
            .await
            .map_err(StatsError::DatabaseError)?;
            
        let mut pending = 0;
        let mut resolved = 0;
        let mut dismissed = 0;
        
        for (status, count) in stats {
            match status {
                ReportStatusEnum::Pending => pending = count,
                ReportStatusEnum::Resolved => resolved = count,
                ReportStatusEnum::Dismissed => dismissed = count,
            }
        }
        
        let total_reports = pending + resolved + dismissed;
        let resolution_rate = if total_reports > 0 {
            Some(((resolved + dismissed) * 100 / total_reports) as i32)
        } else {
            None
        };
        
        let sql = r#"
            SELECT 
                u.username AS nickname, 
                u.avatar_url, 
                u.opt_out_statistics,
                CAST(COUNT(r.id) AS INTEGER) as resolved_count
            FROM users u
            JOIN reports r ON u.id = r.reporter_id
            WHERE r.status = 'resolved'
            GROUP BY u.id, u.username, u.avatar_url, u.opt_out_statistics
            ORDER BY resolved_count DESC
            LIMIT 5
        "#;

        let query = Statement::from_sql_and_values(
            db.get_database_backend(),
            sql,
            vec![]
        );

        let results = db
            .query_all(query)
            .await
            .map_err(StatsError::DatabaseError)?;

        let mut contributors = Vec::new();
        for row in results {
            let opt_out: bool = row.try_get("", "opt_out_statistics").unwrap_or(false);
            
            let nickname: String = if opt_out {
                "Anonim".to_string()
            } else {
                row.try_get("", "nickname").unwrap_or_else(|_| "Bilinmeyen".to_string())
            };
            
            let avatar_url: Option<String> = if opt_out {
                None
            } else {
                row.try_get("", "avatar_url").ok()
            };
            
            let resolved_count: i32 = row.try_get("", "resolved_count").unwrap_or(0);
            
            contributors.push(ContributorDto {
                nickname,
                avatar_url,
                resolved_count,
            });
        }
        
        Ok(HumanityStatsDto {
            resolved_reports: resolved,
            pending_reports: pending,
            total_reports,
            resolution_rate,
            contributors,
        })
    }

    pub async fn get_dish_tags(db: &DatabaseConnection, dish_id: i32) -> Result<Vec<TrendingTagDto>, StatsError> {
        let results = Tags::find()
            .inner_join(dish_tags::Entity)
            .filter(dish_tags::Column::DishId.eq(dish_id))
            .all(db)
            .await
            .map_err(StatsError::DatabaseError)?;
            
        Ok(results.into_iter().map(|t| TrendingTagDto {
            name: t.name,
            count: 1, // Doesn't matter for single dish
            category: t.category,
        }).collect())
    }
}
