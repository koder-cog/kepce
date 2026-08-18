use sea_orm::*;
use shared::entities::{prelude::*, comments, users, vote_reactions, dish_votes};
use uuid::Uuid;
use chrono::Utc;
use std::collections::HashMap;
use crate::dto::comment::{CreateCommentDto, CommentResponseDto, Sentiment as DtoSentiment, ReactionTypeDto};
use crate::services::moderation::{ModerationService, ModerationError};

#[derive(Debug)]
pub enum CommentError {
    UserNotFound,
    UnverifiedUser,
    MenuNotFound,
    DishNotFound,
    DishNotInMenu,
    ParentCommentNotFound,
    InvalidOperation,
    SpamDetected,
    DatabaseError(DbErr),
}

pub struct CommentService;

impl CommentService {
    /// Yeni Yorum Ekleme
    pub async fn create_comment(
        db: &DatabaseConnection,
        user_id: Uuid,
        author_username: String,
        mut dto: CreateCommentDto,
        parent_id: Option<Uuid>,
    ) -> Result<CommentResponseDto, CommentError> {
        // 0. Kullanıcı Varlık Kontrolü
        // JWT geçerli olsa bile kullanıcı hesabını silmiş olabilir.
        let user_opt = Users::find_by_id(user_id)
            .one(db)
            .await
            .map_err(CommentError::DatabaseError)?;
        
        if let Some(u) = user_opt {
            if !u.is_verified {
                return Err(CommentError::UnverifiedUser);
            }
        } else {
            return Err(CommentError::UserNotFound);
        }

        // 1. Menü Kontrolü
        let menu_exists = Menus::find_by_id(dto.menu_id)
            .one(db)
            .await
            .map_err(CommentError::DatabaseError)?
            .is_some();
        if !menu_exists { return Err(CommentError::MenuNotFound); }

        // 2. Tabldot (Yemek) Kontrolü
        // SA-12: is_tabldot is now passed from the frontend explicitly
        let is_tabldot = dto.is_tabldot.unwrap_or(false);
        if let Some(d_id) = dto.dish_id {
            // Önce yemeğin sistemde var olduğunu doğrula
            let dish_exists = shared::entities::dishes::Entity::find_by_id(d_id)
                .one(db)
                .await
                .map_err(CommentError::DatabaseError)?
                .is_some();
            if !dish_exists { return Err(CommentError::DishNotFound); }

            // Sonra o menüde (o gün) çıkıyor mu?
            // Veritabanı düzeyinde filtreleme yaparak RAM israfını önlüyoruz (Inner Join)
            let dish_in_menu = shared::entities::menu_dishes::Entity::find()
                .inner_join(shared::entities::dish_aliases::Entity)
                .filter(shared::entities::menu_dishes::Column::MenuId.eq(dto.menu_id))
                .filter(shared::entities::dish_aliases::Column::DishId.eq(d_id))
                .one(db)
                .await
                .map_err(CommentError::DatabaseError)?
                .is_some();

            if !dish_in_menu { 
                return Err(CommentError::DishNotInMenu); 
            }
        }

        // 3. Yanıt (Reply) Mantıksal Kontrolleri
        if let Some(p_id) = parent_id {
            let parent = Comments::find_by_id(p_id)
                .one(db)
                .await
                .map_err(CommentError::DatabaseError)?
                .ok_or(CommentError::ParentCommentNotFound)?;

            if parent.menu_id != dto.menu_id {
                return Err(CommentError::InvalidOperation);
            }

            if dto.dish_id.is_none() {
                dto.dish_id = parent.dish_id;
            } else if dto.dish_id != parent.dish_id {
                return Err(CommentError::InvalidOperation);
            }

            // Engellenen (veya engelleyen) kişinin yorumuna yanıt verilemez
            if let Some(parent_author_id) = parent.user_id {
                let blocked_user_ids = ModerationService::get_blocked_user_ids(db, user_id).await.map_err(|e| match e {
                    ModerationError::DatabaseError(db_err) => CommentError::DatabaseError(db_err),
                    _ => CommentError::DatabaseError(DbErr::Custom("Moderation error".to_string())),
                })?;
                
                if blocked_user_ids.contains(&parent_author_id) {
                    return Err(CommentError::InvalidOperation);
                }
            }

            // SA-10: Yanıt zinciri derinlik sınırı. Sınırsız derinlik, okuma
            // sırasında özyinelemeli `build_tree`'de stack-overflow DoS'una yol
            // açıyordu. Zincirde yukarı doğru en fazla MAX_REPLY_DEPTH adım çıkılır.
            const MAX_REPLY_DEPTH: usize = 10;
            let mut depth = 1;
            let mut cursor = parent.parent_id;
            while let Some(ancestor_id) = cursor {
                depth += 1;
                if depth >= MAX_REPLY_DEPTH {
                    return Err(CommentError::InvalidOperation); // Zincir çok derin
                }
                cursor = Comments::find_by_id(ancestor_id)
                    .one(db)
                    .await
                    .map_err(CommentError::DatabaseError)?
                    .and_then(|c| c.parent_id);
            }
        }

        // Unified Review Model (URM) Kuralları
        let is_pure_vote = dto.comment.as_ref().map(|s| s.trim().is_empty()).unwrap_or(true);
        
        if is_pure_vote {
            if dto.sentiment == DtoSentiment::Neutral {
                return Err(CommentError::InvalidOperation); // Boş bir oy "Nötr" olamaz.
            }
        } else if !is_tabldot && dto.sentiment != DtoSentiment::Neutral {
            return Err(CommentError::InvalidOperation); // Serbest yorumlar durumu etkileyemez, her zaman "Nötr" olmalıdır.
        }

        let db_sentiment = match dto.sentiment {
            DtoSentiment::Positive => shared::entities::sea_orm_active_enums::SentimentEnum::Positive,
            DtoSentiment::Negative => shared::entities::sea_orm_active_enums::SentimentEnum::Negative,
            DtoSentiment::Neutral => shared::entities::sea_orm_active_enums::SentimentEnum::Neutral,
        };

        // 4. İçerik Moderasyonu (XSS, Spam, Küfür, AI NLP Kontrolü)
        let clean_content = if !is_pure_vote {
            let mut sanitized = shared::services::content_guard::ContentGuard::sanitize_html(dto.comment.as_ref().unwrap());
            
            if shared::services::content_guard::ContentGuard::is_spam(&sanitized) {
                return Err(CommentError::SpamDetected);
            }
            
            // Yerel BERT Toksiklik / Nefret Söylemi Analizi
            if let Ok(ai_res) = ModerationService::check_text_ai(&sanitized).await {
                if ai_res.is_toxic {
                    tracing::warn!(
                        "Yorum yerel BERT NLP tarafından işaretlendi: etiket={}, güven={:.2}",
                        ai_res.label,
                        ai_res.score
                    );
                }
            }

            if shared::services::content_guard::ContentGuard::contains_profanity(&sanitized) {
                sanitized = shared::services::content_guard::ContentGuard::censor_profanity(&sanitized);
            }
            Some(sanitized)
        } else {
            None
        };

        // 5. Modeli Oluştur ve Kaydet
        let new_comment = comments::ActiveModel {
            id: Set(Uuid::new_v4()),
            user_id: Set(Some(user_id)),
            menu_id: Set(dto.menu_id),
            dish_id: Set(dto.dish_id),
            parent_id: Set(parent_id),
            content: Set(clean_content),
            sentiment: Set(db_sentiment.clone()),
            is_tabldot: Set(is_tabldot),
            ..Default::default()
        };

        let txn = db.begin().await.map_err(CommentError::DatabaseError)?;

        let inserted = new_comment.insert(&txn).await.map_err(CommentError::DatabaseError)?;
        
        // 6. Yemek Oylarını (Tabldot) İşle
        if is_tabldot && db_sentiment != shared::entities::sea_orm_active_enums::SentimentEnum::Neutral {
            if let Some(d_id) = dto.dish_id {
                let existing_vote = dish_votes::Entity::find()
                    .filter(dish_votes::Column::DishId.eq(d_id))
                    .filter(dish_votes::Column::MenuId.eq(dto.menu_id))
                    .filter(dish_votes::Column::UserId.eq(user_id))
                    .one(&txn)
                    .await
                    .map_err(CommentError::DatabaseError)?;
                
                if let Some(existing) = existing_vote {
                    let mut active: dish_votes::ActiveModel = existing.into();
                    active.sentiment = Set(db_sentiment.clone());
                    active.is_explicit = Set(true);
                    active.update(&txn).await.map_err(CommentError::DatabaseError)?;
                } else {
                    dish_votes::Entity::insert(dish_votes::ActiveModel {
                        dish_id: Set(d_id),
                        menu_id: Set(dto.menu_id),
                        user_id: Set(user_id),
                        sentiment: Set(db_sentiment.clone()),
                        is_explicit: Set(true),
                        ..Default::default()
                    }).exec(&txn).await.map_err(CommentError::DatabaseError)?;
                }
            }
        }

        txn.commit().await.map_err(CommentError::DatabaseError)?;
        
        // Yanıt bildirimi tetikleyici (Parent yorum sahibi kendisi değilse)
        if let Some(p_id) = parent_id {
            if let Ok(Some(parent_comment)) = Comments::find_by_id(p_id).one(db).await {
                if let Some(parent_author_id) = parent_comment.user_id {
                    if parent_author_id != user_id {
                        let preview = inserted.content.as_deref().unwrap_or("Bir yanıt bıraktı");
                        let truncated = if preview.chars().count() > 80 {
                            format!("{}...", preview.chars().take(77).collect::<String>())
                        } else {
                            preview.to_string()
                        };
                        let action_href = format!("/yorumlar/{}?thread={}", dto.menu_id, p_id);
                        let _ = crate::services::notification::NotificationService::send_notification(
                            db,
                            parent_author_id,
                            "reply",
                            &format!("@{} yorumuna yanıt verdi", author_username),
                            &truncated,
                            Some("Yanıta Git"),
                            Some(&action_href),
                        ).await;
                    }
                }
            }
        }

        Ok(CommentResponseDto {
            id: inserted.id,
            comment: inserted.content,
            sentiment: dto.sentiment,
            is_tabldot: inserted.is_tabldot,
            user: crate::dto::comment::UserSummaryDto { id: user_id, nickname: author_username.clone(), avatar_url: None },
            reaction_summary: crate::dto::comment::ReactionSummaryDto { up: 0, down: 0, my_vote: None },
            children: vec![],
            created_at: inserted.created_at.unwrap_or_else(|| Utc::now().into()).into(),
            is_deleted: false,
            deletion_type: None,
        })
    }


    /// Yorum verilerini (Kullanıcı, Oylar, Engeller) zenginleştiren merkezi fonksiyon (DRY)
    pub async fn enrich_comments(
        db: &DatabaseConnection,
        results: Vec<(comments::Model, Option<users::Model>)>,
        current_user_id: Option<Uuid>,
        blocked_user_ids: &[Uuid],
    ) -> Result<Vec<(CommentResponseDto, Option<Uuid>)>, CommentError> {
        
        if results.is_empty() {
            return Ok(vec![]);
        }

        let comment_ids: Vec<Uuid> = results.iter().map(|(c, _)| c.id).collect();

        // 1. Tüm reaksiyonları tek seferde çek
        let reactions = VoteReactions::find()
            .filter(vote_reactions::Column::CommentId.is_in(comment_ids))
            .all(db)
            .await
            .map_err(CommentError::DatabaseError)?;

        // Reaksiyonları CommentID'ye göre grupla
        let mut reaction_map: HashMap<Uuid, (i32, i32, Option<ReactionTypeDto>)> = HashMap::new();
        for r in reactions {
            let entry = reaction_map.entry(r.comment_id).or_insert((0, 0, None));
            let is_mine = current_user_id == Some(r.user_id);
            
            match r.reaction_type {
                shared::entities::sea_orm_active_enums::ReactionTypeEnum::Upvote => {
                    entry.0 += 1;
                    if is_mine { entry.2 = Some(ReactionTypeDto::Up); }
                }
                shared::entities::sea_orm_active_enums::ReactionTypeEnum::Downvote => {
                    entry.1 += 1;
                    if is_mine { entry.2 = Some(ReactionTypeDto::Down); }
                }
            }
        }

        // 2. Düz Listeyi oluştur
        let mut enriched_list = Vec::new();

        for (comment, user_opt) in results {
            let _author_id = comment.user_id;
            let author_username = user_opt.as_ref().map(|u| u.username.clone()).unwrap_or_else(|| "Bilinmeyen Kullanıcı".to_string());
            let mut avatar_url = user_opt.and_then(|u| u.avatar_url);

            // Silinen veya maskelenen içerik kontrolü
            let is_deleted = comment.is_deleted || author_username == "Bilinmeyen Kullanıcı";
            let is_blocked = comment.user_id.map(|uid| blocked_user_ids.contains(&uid)).unwrap_or(false);

            let content = if is_deleted {
                Some("[Bu içerik silinmiş]".to_string())
            } else if is_blocked {
                Some("[Bu içerik görüntülenemiyor]".to_string())
            } else {
                comment.content.clone()
            };

            let author_username = if is_deleted {
                "Bilinmeyen Kullanıcı".to_string()
            } else if is_blocked {
                "Gizli Kullanıcı".to_string()
            } else {
                author_username
            };

            if is_deleted || is_blocked {
                avatar_url = None;
            }

            let user_dto_id = if is_deleted {
                Uuid::nil()
            } else {
                comment.user_id.unwrap_or_default()
            };

            let dto_sentiment = match comment.sentiment {
                shared::entities::sea_orm_active_enums::SentimentEnum::Positive => DtoSentiment::Positive,
                shared::entities::sea_orm_active_enums::SentimentEnum::Negative => DtoSentiment::Negative,
                shared::entities::sea_orm_active_enums::SentimentEnum::Neutral => DtoSentiment::Neutral,
            };

            let (upvotes, downvotes, my_vote) = reaction_map.get(&comment.id).cloned().unwrap_or((0, 0, None));

            let dto = CommentResponseDto {
                id: comment.id,
                comment: content,
                sentiment: dto_sentiment,
                is_tabldot: comment.is_tabldot,
                user: crate::dto::comment::UserSummaryDto { id: user_dto_id, nickname: author_username.clone(), avatar_url },
                reaction_summary: crate::dto::comment::ReactionSummaryDto { up: upvotes, down: downvotes, my_vote },
                children: vec![],
                created_at: comment.created_at.unwrap_or_else(|| Utc::now().into()).into(),
                is_deleted,
                deletion_type: comment.deletion_type.clone(),
            };

            enriched_list.push((dto, comment.parent_id));
        }

        Ok(enriched_list)
    }

    /// Menüye ait yorumları Ağaç (Tree) formatında zenginleştirip döner
    pub async fn get_menu_comment_tree(
        db: &DatabaseConnection,
        menu_id: i32,
        current_user_id: Option<Uuid>,
        limit: Option<u64>,
        offset: Option<u64>,
    ) -> Result<Vec<CommentResponseDto>, CommentError> {
        
        let blocked_user_ids = if let Some(uid) = current_user_id {
            ModerationService::get_blocked_user_ids(db, uid).await.map_err(|e| match e {
                ModerationError::DatabaseError(db_err) => CommentError::DatabaseError(db_err),
                _ => CommentError::DatabaseError(DbErr::Custom("Moderation error".to_string())),
            })?
        } else {
            vec![]
        };

        let query = Comments::find().filter(comments::Column::MenuId.eq(menu_id));

        let results = query
            .find_also_related(shared::entities::users::Entity)
            .order_by_asc(comments::Column::CreatedAt)
            .all(db)
            .await
            .map_err(CommentError::DatabaseError)?;

        let enriched_flat = Self::enrich_comments(db, results, current_user_id, &blocked_user_ids).await?;

        // HashMap tabanlı lookup için DTO'ları ID ile tutalım
        let mut flat_comments: HashMap<Uuid, (CommentResponseDto, Option<Uuid>)> = HashMap::new();
        for (ref dto, parent_id) in &enriched_flat {
            flat_comments.insert(dto.id, (dto.clone(), *parent_id));
        }

        // Ağaç (Tree) Yapısını Kur ve Yetim (Orphan) Yorumları Kurtar
        let mut root_ids = Vec::new();
        let mut child_map: HashMap<Uuid, Vec<Uuid>> = HashMap::new();

        // Kök yorumların kronolojik sıralamasını korumak için enriched_flat üzerinden dönüyoruz
        for (dto, parent_id_opt) in &enriched_flat {
            let id = dto.id;
            match parent_id_opt {
                Some(pid) if flat_comments.contains_key(pid) => {
                    child_map.entry(*pid).or_default().push(id);
                }
                _ => {
                    root_ids.push(id);
                }
            }
        }

        // Kök yorumlar için limit ve offset sayfalama kurallarını uygula
        let total_roots = root_ids.len();
        let page_limit = limit.unwrap_or(100).min(500) as usize;
        let page_offset = offset.unwrap_or(0) as usize;

        let paginated_root_ids = if page_offset < total_roots {
            let end = (page_offset + page_limit).min(total_roots);
            root_ids[page_offset..end].to_vec()
        } else {
            vec![]
        };

        fn build_tree(
            id: &Uuid,
            flat_map: &HashMap<Uuid, (CommentResponseDto, Option<Uuid>)>,
            child_map: &HashMap<Uuid, Vec<Uuid>>
        ) -> Option<CommentResponseDto> {
            let mut node = flat_map.get(id)?.0.clone();
            if let Some(children) = child_map.get(id) {
                for child_id in children {
                    if let Some(child_node) = build_tree(child_id, flat_map, child_map) {
                        node.children.push(child_node);
                    }
                }
            }
            Some(node)
        }

        let mut tree = Vec::new();
        for root_id in paginated_root_ids {
            if let Some(root_node) = build_tree(&root_id, &flat_comments, &child_map) {
                tree.push(root_node);
            }
        }

        Ok(tree)
    }

    /// Bir kullanıcının profili için attığı tüm yorumları (flat formatta) getirir
    pub async fn get_user_comments(
        db: &DatabaseConnection,
        target_user_id: Uuid,
        current_user_id: Option<Uuid>,
        limit: u64,
        offset: u64,
    ) -> Result<crate::dto::pagination::PaginatedResponse<CommentResponseDto>, CommentError> {
        let base_query = Comments::find().filter(comments::Column::UserId.eq(target_user_id));
        let total_items = base_query.clone().count(db).await.map_err(CommentError::DatabaseError)?;

        let results = base_query
            .find_also_related(shared::entities::users::Entity)
            .order_by_desc(comments::Column::CreatedAt)
            .limit(limit)
            .offset(offset)
            .all(db)
            .await
            .map_err(CommentError::DatabaseError)?;

        let blocked_user_ids = if let Some(uid) = current_user_id {
            ModerationService::get_blocked_user_ids(db, uid).await.map_err(|e| match e {
                ModerationError::DatabaseError(db_err) => CommentError::DatabaseError(db_err),
                _ => CommentError::DatabaseError(DbErr::Custom("Moderation error".to_string())),
            })?
        } else {
            vec![]
        };

        let enriched = Self::enrich_comments(db, results, current_user_id, &blocked_user_ids).await?;
        
        let items: Vec<_> = enriched.into_iter().map(|(dto, _)| dto).collect();
        
        Ok(crate::dto::pagination::PaginatedResponse {
            items,
            total_items,
            total_pages: (total_items as f64 / limit as f64).ceil() as u64,
            current_page: (offset / limit) + 1,
        })
    }

    /// İstatistikler veya akış için en son atılan yorumları getirir
    pub async fn get_recent_comments(
        db: &DatabaseConnection,
        current_user_id: Option<Uuid>,
        limit: u64,
    ) -> Result<Vec<CommentResponseDto>, CommentError> {
        let blocked_user_ids = if let Some(uid) = current_user_id {
            ModerationService::get_blocked_user_ids(db, uid).await.map_err(|e| match e {
                ModerationError::DatabaseError(db_err) => CommentError::DatabaseError(db_err),
                _ => CommentError::DatabaseError(DbErr::Custom("Moderation error".to_string())),
            })?
        } else {
            vec![]
        };

        let query = Comments::find();

        let results = query
            .find_also_related(shared::entities::users::Entity)
            .order_by_desc(comments::Column::CreatedAt)
            .limit(limit)
            .all(db)
            .await
            .map_err(CommentError::DatabaseError)?;

        let enriched = Self::enrich_comments(db, results, current_user_id, &blocked_user_ids).await?;
        
        Ok(enriched.into_iter().map(|(dto, _)| dto).collect())
    }

    /// İstatistikler için en çok upvote alan yorumları getirir
    pub async fn get_top_comments(
        db: &DatabaseConnection,
        current_user_id: Option<Uuid>,
        limit: u64,
        timeframe: Option<String>,
    ) -> Result<Vec<CommentResponseDto>, CommentError> {
        let blocked_user_ids = if let Some(uid) = current_user_id {
            ModerationService::get_blocked_user_ids(db, uid).await.map_err(|e| match e {
                ModerationError::DatabaseError(db_err) => CommentError::DatabaseError(db_err),
                _ => CommentError::DatabaseError(DbErr::Custom("Moderation error".to_string())),
            })?
        } else {
            vec![]
        };

        let mut query = Comments::find();

        if let Some(tf) = timeframe {
            let interval = match tf.as_str() {
                "daily" => "1 day",
                "weekly" => "7 days",
                "monthly" => "30 days",
                "yearly" => "365 days",
                _ => "",
            };
            if !interval.is_empty() {
                // SeaORM raw filter for created_at
                query = query.filter(sea_orm::sea_query::Expr::cust(format!("created_at >= NOW() - INTERVAL '{}'", interval).as_str()));
            }
        }

        let results = query
            .find_also_related(shared::entities::users::Entity)
            .order_by_desc(comments::Column::CreatedAt)
            .limit(limit * 5) 
            .all(db)
            .await
            .map_err(CommentError::DatabaseError)?;

        let mut enriched = Self::enrich_comments(db, results, current_user_id, &blocked_user_ids).await?;
        
        // Sort by score
        enriched.sort_by(|a, b| {
            let a_score = a.0.reaction_summary.up - a.0.reaction_summary.down;
            let b_score = b.0.reaction_summary.up - b.0.reaction_summary.down;
            b_score.cmp(&a_score)
        });
        
        Ok(enriched.into_iter().take(limit as usize).map(|(dto, _)| dto).collect())
    }
}
