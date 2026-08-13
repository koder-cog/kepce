use sea_orm::*;
use shared::entities::{prelude::*, comments, vote_reactions};
use uuid::Uuid;
use crate::dto::comment::ReactionTypeDto;
use crate::dto::reaction::ReactionSummaryDto;

#[derive(Debug)]
pub enum ReactionError {
    CommentNotFound,
    Unauthorized,
    UnverifiedUser,
    InvalidOperation,
    DatabaseError(DbErr),
}

pub struct ReactionService;

impl ReactionService {
    /// Yoruma verilen oyları değiştirir veya geri alır (Toggle mantığı).
    /// Geriye anlık oylama durumunu döndürür.
    pub async fn toggle_reaction(
        db: &DatabaseConnection,
        user_id: Uuid,
        comment_id: Uuid,
        reaction_type: ReactionTypeDto,
    ) -> Result<ReactionSummaryDto, ReactionError> {
        // Kullanıcı Onay Kontrolü
        let user = Users::find_by_id(user_id)
            .one(db)
            .await
            .map_err(ReactionError::DatabaseError)?
            .ok_or(ReactionError::Unauthorized)?;

        if !user.is_verified {
            return Err(ReactionError::UnverifiedUser);
        }
        
        // 1. Yorumun varlığını doğrula
        let comment = Comments::find_by_id(comment_id)
            .one(db)
            .await
            .map_err(ReactionError::DatabaseError)?
            .ok_or(ReactionError::CommentNotFound)?;

        // Engellenen kullanıcı (veya bizi engelleyen kullanıcı) yorumuna oy verilemez
        if let Some(author_id) = comment.user_id {
            let blocked_user_ids = crate::services::moderation::ModerationService::get_blocked_user_ids(db, user_id)
                .await
                .map_err(|e| match e {
                    crate::services::moderation::ModerationError::DatabaseError(db_err) => ReactionError::DatabaseError(db_err),
                    _ => ReactionError::DatabaseError(DbErr::Custom("Moderation error".to_string())),
                })?;
            
            if blocked_user_ids.contains(&author_id) {
                return Err(ReactionError::InvalidOperation);
            }
        }

        // 2. Kullanıcının bu yoruma daha önce verdiği bir oy var mı?
        let existing_reaction = VoteReactions::find()
            .filter(vote_reactions::Column::UserId.eq(user_id))
            .filter(vote_reactions::Column::CommentId.eq(comment_id))
            .one(db)
            .await
            .map_err(ReactionError::DatabaseError)?;

        let target_db_enum = match reaction_type {
            ReactionTypeDto::Up => shared::entities::sea_orm_active_enums::ReactionTypeEnum::Upvote,
            ReactionTypeDto::Down => shared::entities::sea_orm_active_enums::ReactionTypeEnum::Downvote,
        };

        if let Some(reaction) = existing_reaction {
            if reaction.reaction_type == target_db_enum {
                // Senaryo A: Aynı oya tekrar tıklandı -> Oyu kaldır (Toggle Off)
                reaction.delete(db).await.map_err(ReactionError::DatabaseError)?;
            } else {
                // Senaryo B: Farklı oya tıklandı (Upvote -> Downvote) -> Oyu güncelle
                let mut active_reaction: vote_reactions::ActiveModel = reaction.into();
                active_reaction.reaction_type = Set(target_db_enum);
                active_reaction.update(db).await.map_err(ReactionError::DatabaseError)?;
            }
        } else {
            // Senaryo C: İlk defa oy veriliyor -> Yeni ekle
            let new_reaction = vote_reactions::ActiveModel {
                user_id: Set(user_id),
                comment_id: Set(comment_id),
                reaction_type: Set(target_db_enum),
                ..Default::default()
            };
            
            // Eğer tam bu satırda race condition oluşur ve aynı anda 2 insert gelirse,
            // utils::db::is_unique_constraint_violation ile kontrol eklenebilir.
            // Fakat toggle işlemi genelde tek client'tan sıralı gelir.
            new_reaction.insert(db).await.map_err(ReactionError::DatabaseError)?;
        }

        // 3. Güncel Oylama Durumunu (Summary) Çek
        Self::get_reaction_summary(db, comment_id, Some(user_id)).await
    }

    /// Bir yorumu siler. Fiziksel silme yerine yorumun user_id alanını NULL yapar.
    /// Böylece alt yorumlar (replies) yetim kalmaz, ağaç bozulmaz.
    /// Yorumun içeriği "[Bu içerik silinmiş]" olarak CommentService tarafından otomatik maskelenir.
    pub async fn delete_comment(
        db: &DatabaseConnection,
        user_id: Uuid,
        user_role: &crate::dto::user::UserRole,
        comment_id: Uuid,
    ) -> Result<(), ReactionError> {
        let comment = Comments::find_by_id(comment_id)
            .one(db)
            .await
            .map_err(ReactionError::DatabaseError)?
            .ok_or(ReactionError::CommentNotFound)?;

        // Admin her yorumu silebilir (moderasyon). Normal kullanıcı sadece kendisininkileri.
        let is_admin = *user_role == crate::dto::user::UserRole::Admin;
        if !is_admin && comment.user_id != Some(user_id) {
            return Err(ReactionError::Unauthorized);
        }

        let mut active_comment: comments::ActiveModel = comment.into();
        active_comment.user_id = Set(None); // Soft delete (Ağaç yapısını korumak için)
        active_comment.is_deleted = Set(true);
        active_comment.deletion_type = Set(Some(if is_admin { "admin".to_string() } else { "user".to_string() }));
        
        // KVKK İhlalini Önlemek:
        // Sadece yazar kimliğini gizlemek yetmez, veritabanındaki asıl metni de
        // fiziksel olarak yok etmeliyiz ki içerisinde kalmış olabilecek
        // telefon numarası, hakaret veya kişisel veriler tamamen silinsin.
        active_comment.content = Set(Some("[Bu içerik silinmiş]".to_string()));
        
        active_comment.update(db).await.map_err(ReactionError::DatabaseError)?;

        Ok(())
    }

    /// Özel bir yorumun güncel oylama durumunu döner.
    pub async fn get_reaction_summary(
        db: &DatabaseConnection,
        comment_id: Uuid,
        current_user_id: Option<Uuid>,
    ) -> Result<ReactionSummaryDto, ReactionError> {
        
        // 3 kere veritabanına gidip gelmek (round-trip) yerine
        // sadece kullanıcının oyunu ve genel aggregate datayı çekecek 2 turlu veya
        // Custom struct ile tek turlu query atabiliriz. SeaORM'in karmaşıklığından kaçınmak
        // ve 3 gidiş-dönüşü 1+1'e indirmek için GROUP BY kullanıyoruz.
        
        let counts = VoteReactions::find()
            .filter(vote_reactions::Column::CommentId.eq(comment_id))
            .select_only()
            .column(vote_reactions::Column::ReactionType)
            .column_as(sea_orm::sea_query::Expr::col(vote_reactions::Column::Id).count(), "count")
            .group_by(vote_reactions::Column::ReactionType)
            .into_tuple::<(shared::entities::sea_orm_active_enums::ReactionTypeEnum, i64)>()
            .all(db)
            .await
            .map_err(ReactionError::DatabaseError)?;

        let mut upvotes = 0;
        let mut downvotes = 0;

        for (rtype, count) in counts {
            match rtype {
                shared::entities::sea_orm_active_enums::ReactionTypeEnum::Upvote => upvotes = count as i32,
                shared::entities::sea_orm_active_enums::ReactionTypeEnum::Downvote => downvotes = count as i32,
            }
        }

        let mut my_vote = None;
        if let Some(uid) = current_user_id {
            // My_vote için mecburen 2. bir çok hafif query atıyoruz.
            // Bu sayede 3 yerine 2 round-trip ile hem RAM'i hem Network'ü koruduk.
            let user_reaction = VoteReactions::find()
                .filter(vote_reactions::Column::CommentId.eq(comment_id))
                .filter(vote_reactions::Column::UserId.eq(uid))
                .one(db)
                .await
                .map_err(ReactionError::DatabaseError)?;

            if let Some(r) = user_reaction {
                my_vote = Some(match r.reaction_type {
                    shared::entities::sea_orm_active_enums::ReactionTypeEnum::Upvote => ReactionTypeDto::Up,
                    shared::entities::sea_orm_active_enums::ReactionTypeEnum::Downvote => ReactionTypeDto::Down,
                });
            }
        }

        Ok(ReactionSummaryDto {
            upvotes,
            downvotes,
            my_vote,
        })
    }
}
