use sea_orm::*;
use uuid::Uuid;
use chrono::{Utc, Duration};
use std::collections::BTreeMap;
use sha2::{Sha256, Digest};
use rand::{thread_rng, Rng};
use shared::entities::{
    prelude::*, projects, api_keys, api_usage_logs
};
use crate::dto::developer::{
    ProjectResponseDto, ApiKeyResponseDto, ApiUsageDto
};

#[derive(Debug)]
pub enum DeveloperError {
    NotFound,
    Unauthorized,
    UnverifiedUser,
    DatabaseError(DbErr),
    InvalidInput(String),
}

impl From<DbErr> for DeveloperError {
    fn from(err: DbErr) -> Self {
        DeveloperError::DatabaseError(err)
    }
}

pub struct DeveloperService;

impl DeveloperService {
    pub async fn get_projects(
        db: &DatabaseConnection,
        user_id: Uuid,
    ) -> Result<Vec<ProjectResponseDto>, DeveloperError> {
        let list = Projects::find()
            .filter(projects::Column::UserId.eq(user_id))
            .all(db)
            .await?;

        Ok(list.into_iter().map(|p| ProjectResponseDto {
            id: p.id,
            name: p.name,
            created_at: p.created_at.map(|t| t.into()),
            updated_at: p.updated_at.map(|t| t.into()),
        }).collect())
    }

    pub async fn create_project(
        db: &DatabaseConnection,
        user_id: Uuid,
        name: String,
    ) -> Result<ProjectResponseDto, DeveloperError> {
        let user = Users::find_by_id(user_id)
            .one(db)
            .await?
            .ok_or(DeveloperError::Unauthorized)?;

        if !user.is_verified {
            return Err(DeveloperError::UnverifiedUser);
        }

        if name.trim().is_empty() {
            return Err(DeveloperError::InvalidInput("Proje ismi boş olamaz".to_string()));
        }

        let new_proj = projects::ActiveModel {
            id: Set(Uuid::new_v4()),
            user_id: Set(user_id),
            name: Set(name),
            ..Default::default()
        };

        let inserted = new_proj.insert(db).await?;

        Ok(ProjectResponseDto {
            id: inserted.id,
            name: inserted.name,
            created_at: inserted.created_at.map(|t| t.into()),
            updated_at: inserted.updated_at.map(|t| t.into()),
        })
    }

    pub async fn update_project(
        db: &DatabaseConnection,
        user_id: Uuid,
        project_id: Uuid,
        name: String,
    ) -> Result<ProjectResponseDto, DeveloperError> {
        if name.trim().is_empty() {
            return Err(DeveloperError::InvalidInput("Proje ismi boş olamaz".to_string()));
        }

        let project = Projects::find_by_id(project_id)
            .one(db)
            .await?
            .ok_or(DeveloperError::NotFound)?;

        if project.user_id != user_id {
            return Err(DeveloperError::Unauthorized);
        }

        let mut active: projects::ActiveModel = project.into();
        active.name = Set(name);
        active.updated_at = Set(Some(chrono::Utc::now().into()));

        let updated = active.update(db).await?;

        Ok(ProjectResponseDto {
            id: updated.id,
            name: updated.name,
            created_at: updated.created_at.map(|t| t.into()),
            updated_at: updated.updated_at.map(|t| t.into()),
        })
    }

    pub async fn delete_project(
        db: &DatabaseConnection,
        user_id: Uuid,
        project_id: Uuid,
    ) -> Result<(), DeveloperError> {
        let project = Projects::find_by_id(project_id)
            .one(db)
            .await?
            .ok_or(DeveloperError::NotFound)?;

        if project.user_id != user_id {
            return Err(DeveloperError::Unauthorized);
        }

        project.delete(db).await?;
        Ok(())
    }

    pub async fn get_api_keys(
        db: &DatabaseConnection,
        user_id: Uuid,
    ) -> Result<Vec<ApiKeyResponseDto>, DeveloperError> {
        let keys = ApiKeys::find()
            .filter(api_keys::Column::UserId.eq(user_id))
            .all(db)
            .await?;

        Ok(keys.into_iter().map(|k| ApiKeyResponseDto {
            id: k.id,
            project_id: k.project_id,
            name: k.name,
            key_prefix: k.key_prefix,
            is_active: k.is_active,
            created_at: k.created_at.map(|t| t.into()),
            updated_at: k.updated_at.map(|t| t.into()),
            key: None,
        }).collect())
    }

    pub async fn create_api_key(
        db: &DatabaseConnection,
        user_id: Uuid,
        project_id: Uuid,
        name: String,
    ) -> Result<ApiKeyResponseDto, DeveloperError> {
        let user = Users::find_by_id(user_id)
            .one(db)
            .await?
            .ok_or(DeveloperError::Unauthorized)?;

        if !user.is_verified {
            return Err(DeveloperError::UnverifiedUser);
        }

        // Validate project ownership
        let project = Projects::find_by_id(project_id)
            .one(db)
            .await?
            .ok_or(DeveloperError::NotFound)?;

        if project.user_id != user_id {
            return Err(DeveloperError::Unauthorized);
        }

        // Generate key
        let mut random_bytes = [0u8; 16];
        thread_rng().fill(&mut random_bytes);
        let random_hex: String = random_bytes.iter().map(|b| format!("{:02x}", b)).collect();
        let raw_key = format!("kp_live_{}", random_hex); // length = 8 + 32 = 40 chars

        // Prefix is the first 10 characters (i.e. kp_live_xx)
        let key_prefix = raw_key[0..10].to_string();

        // SHA-256 hash the raw key
        let mut hasher = Sha256::new();
        hasher.update(raw_key.as_bytes());
        let hash_result = hasher.finalize();
        let key_hash: String = hash_result.iter().map(|b| format!("{:02x}", b)).collect();

        let new_key = api_keys::ActiveModel {
            id: Set(Uuid::new_v4()),
            project_id: Set(project_id),
            user_id: Set(user_id),
            key_hash: Set(key_hash),
            key_prefix: Set(key_prefix),
            name: Set(name),
            tier: Set("free".to_string()),
            is_active: Set(true),
            ..Default::default()
        };

        let inserted = new_key.insert(db).await?;

        Ok(ApiKeyResponseDto {
            id: inserted.id,
            project_id: inserted.project_id,
            name: inserted.name,
            key_prefix: inserted.key_prefix,
            is_active: inserted.is_active,
            created_at: inserted.created_at.map(|t| t.into()),
            updated_at: inserted.updated_at.map(|t| t.into()),
            key: Some(raw_key),
        })
    }

    pub async fn revoke_api_key(
        db: &DatabaseConnection,
        user_id: Uuid,
        key_id: Uuid,
    ) -> Result<(), DeveloperError> {
        let key = ApiKeys::find_by_id(key_id)
            .one(db)
            .await?
            .ok_or(DeveloperError::NotFound)?;

        if key.user_id != user_id {
            return Err(DeveloperError::Unauthorized);
        }

        key.delete(db).await?;
        Ok(())
    }

    pub async fn get_api_usage(
        db: &DatabaseConnection,
        user_id: Uuid,
        project_id: &str,
        days: i32,
    ) -> Result<Vec<ApiUsageDto>, DeveloperError> {
        let today = Utc::now().date_naive();
        let start_date = today - Duration::days((days.max(1) as i64) - 1);

        // Prepopulate a BTreeMap with 0 values for each day in range
        let mut usage_map = BTreeMap::new();
        let mut current_date = start_date;
        while current_date <= today {
            usage_map.insert(current_date, (0, 0));
            current_date = current_date.succ_opt().unwrap_or(current_date);
        }

        // Check project if it is specific
        if project_id != "all" {
            let proj_uuid = Uuid::parse_str(project_id)
                .map_err(|_| DeveloperError::InvalidInput("Geçersiz proje ID".to_string()))?;
            let project = Projects::find_by_id(proj_uuid)
                .one(db)
                .await?
                .ok_or(DeveloperError::NotFound)?;
            if project.user_id != user_id {
                return Err(DeveloperError::Unauthorized);
            }
        }

        // Query api_usage_logs
        let mut query = ApiUsageLogs::find()
            .join(JoinType::InnerJoin, api_usage_logs::Relation::ApiKeys.def())
            .join(JoinType::InnerJoin, api_keys::Relation::Projects.def())
            .filter(projects::Column::UserId.eq(user_id))
            .filter(api_usage_logs::Column::Date.gte(start_date))
            .filter(api_usage_logs::Column::Date.lte(today));

        if project_id != "all" {
            let proj_uuid = Uuid::parse_str(project_id)
                .map_err(|_| DeveloperError::InvalidInput("Geçersiz proje ID".to_string()))?;
            query = query.filter(projects::Column::Id.eq(proj_uuid));
        }

        let logs = query.all(db).await?;

        for log in logs {
            if let Some(val) = usage_map.get_mut(&log.date) {
                val.0 += log.requests;
                val.1 += log.errors;
            }
        }

        let result = usage_map
            .into_iter()
            .map(|(date, (requests, errors))| ApiUsageDto {
                date: date.to_string(),
                requests,
                errors,
            })
            .collect();

        Ok(result)
    }
}
